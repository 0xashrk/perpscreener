use crate::models::{
    current_ts_seconds, Direction, EquitySnapshot, Position, PositionDraft, RecipeConfig,
    ResumeState, SignalRecord, CHECKPOINT_EVERY,
};
use anyhow::{anyhow, Context, Result};
use rusqlite::{params, Connection};
use serde_json::json;
use std::fs;
use std::path::Path;
use std::time::Duration;

pub struct Db {
    conn: Connection,
    write_count: usize,
}

impl Db {
    pub fn new(path: &str) -> Result<Self> {
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent).context("failed to create db directory")?;
        }

        let conn = Connection::open(path).context("failed to open database")?;
        conn.busy_timeout(Duration::from_secs(5))
            .context("failed to set busy_timeout")?;
        conn.pragma_update(None, "journal_mode", "wal")
            .context("failed to enable WAL")?;

        let db = Self {
            conn,
            write_count: 0,
        };
        db.ensure_schema()?;
        Ok(db)
    }

    pub fn insert_run(
        &mut self,
        run_key: i64,
        coin: &str,
        initial_capital: f64,
        backend: &str,
        interval_secs: u64,
        duration_secs: Option<u64>,
        recipe: &RecipeConfig,
    ) -> Result<()> {
        let config_json = json!({
            "coin": coin,
            "capital": initial_capital,
            "backend": backend,
            "interval_secs": interval_secs,
            "duration_secs": duration_secs,
            "recipe": recipe,
        })
        .to_string();

        self.conn.execute(
            "
            INSERT OR IGNORE INTO runs (start_ts, coin, initial_capital, config)
            VALUES (?1, ?2, ?3, ?4)
            ",
            params![run_key, coin, initial_capital, config_json],
        )?;
        self.bump_writes()?;
        Ok(())
    }

    pub fn update_run_final(&mut self, run_key: i64, final_capital: f64) -> Result<()> {
        let end_ts = current_ts_seconds();
        self.conn.execute(
            "UPDATE runs SET end_ts = ?1, final_capital = ?2 WHERE start_ts = ?3",
            params![end_ts, final_capital, run_key],
        )?;
        self.bump_writes()?;
        Ok(())
    }

    pub fn log_signal(&mut self, rec: &SignalRecord) -> Result<()> {
        self.conn.execute(
            "
            INSERT INTO signals (
                run_key, ts, coin, price, bid, ask, spread, ob_imbalance,
                last_close, prev_close, momentum, signal, reason, position_open
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
                ?9, ?10, ?11, ?12, ?13, ?14
            )
            ",
            params![
                rec.run_key,
                rec.ts,
                rec.coin,
                rec.price,
                rec.bid,
                rec.ask,
                rec.spread,
                rec.ob_imbalance,
                rec.last_close,
                rec.prev_close,
                rec.momentum,
                rec.signal,
                rec.reason,
                if rec.position_open { 1 } else { 0 }
            ],
        )?;
        self.bump_writes()?;
        Ok(())
    }

    pub fn insert_trade_entry(
        &mut self,
        run_key: i64,
        coin: &str,
        position: &PositionDraft,
    ) -> Result<i64> {
        self.conn.execute(
            "
            INSERT INTO trades (
                run_key, coin, direction, entry_ts, entry_px, size, notional,
                entry_fee
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ",
            params![
                run_key,
                coin,
                position.direction.as_str(),
                position.entry_ts,
                position.entry_px,
                position.size_coins,
                position.notional,
                position.entry_fee
            ],
        )?;
        self.bump_writes()?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_trade_exit(
        &mut self,
        trade_id: i64,
        exit_ts: i64,
        exit_px: f64,
        gross_pnl: f64,
        entry_fee: f64,
        exit_fee: f64,
        net_pnl: f64,
        exit_reason: &str,
    ) -> Result<()> {
        self.conn.execute(
            "
            UPDATE trades SET
                exit_ts = ?1,
                exit_px = ?2,
                gross_pnl = ?3,
                entry_fee = ?4,
                exit_fee = ?5,
                net_pnl = ?6,
                exit_reason = ?7
            WHERE id = ?8
            ",
            params![
                exit_ts,
                exit_px,
                gross_pnl,
                entry_fee,
                exit_fee,
                net_pnl,
                exit_reason,
                trade_id
            ],
        )?;
        self.bump_writes()?;
        Ok(())
    }

    pub fn insert_equity(&mut self, snapshot: &EquitySnapshot) -> Result<()> {
        self.conn.execute(
            "
            INSERT OR REPLACE INTO equity (
                run_key, ts, capital, unrealized_pnl, realized_pnl,
                total_trades, win_count, loss_count
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ",
            params![
                snapshot.run_key,
                snapshot.ts,
                snapshot.capital,
                snapshot.unrealized_pnl,
                snapshot.realized_pnl,
                snapshot.total_trades,
                snapshot.win_count,
                snapshot.loss_count
            ],
        )?;
        self.bump_writes()?;
        Ok(())
    }

    pub fn load_resume_state(&self) -> Result<Option<ResumeState>> {
        let mut stmt = self.conn.prepare(
            "
            SELECT id, run_key, direction, entry_ts, entry_px, size, notional, entry_fee
            FROM trades
            WHERE exit_ts IS NULL
            ORDER BY entry_ts DESC
            LIMIT 1
            ",
        )?;

        let mut rows = stmt.query([])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };

        let run_key: i64 = row.get(1)?;
        let direction_str: String = row.get(2)?;
        let direction = Direction::from_str(&direction_str)
            .ok_or_else(|| anyhow!("unknown direction in db: {}", direction_str))?;
        let position = Position {
            db_id: row.get(0)?,
            direction,
            entry_ts: row.get(3)?,
            entry_px: row.get(4)?,
            size_coins: row.get(5)?,
            notional: row.get(6)?,
            entry_fee: row.get(7)?,
        };

        let mut equity_stmt = self.conn.prepare(
            "
            SELECT capital, realized_pnl, total_trades, win_count, loss_count
            FROM equity
            WHERE run_key = ?1
            ORDER BY ts DESC
            LIMIT 1
            ",
        )?;
        let equity_row = equity_stmt.query_row(params![run_key], |r| {
            Ok((
                r.get::<_, f64>(0)?,
                r.get::<_, f64>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, i64>(4)?,
            ))
        });

        let (capital, realized_pnl, total_trades, win_count, loss_count) = match equity_row {
            Ok(vals) => vals,
            Err(_) => {
                let realized_pnl: f64 = self.conn.query_row(
                    "
                    SELECT COALESCE(SUM(net_pnl), 0.0) FROM trades
                    WHERE run_key = ?1 AND exit_ts IS NOT NULL
                    ",
                    params![run_key],
                    |r| r.get(0),
                )?;

                let (total_trades, win_count, loss_count) = self.conn.query_row(
                    "
                    SELECT
                        COUNT(*) as total_trades,
                        SUM(CASE WHEN net_pnl > 0 THEN 1 ELSE 0 END) as wins,
                        SUM(CASE WHEN net_pnl <= 0 THEN 1 ELSE 0 END) as losses
                    FROM trades
                    WHERE run_key = ?1 AND exit_ts IS NOT NULL
                    ",
                    params![run_key],
                    |r| {
                        Ok((
                            r.get::<_, i64>(0)?,
                            r.get::<_, i64>(1)?,
                            r.get::<_, i64>(2)?,
                        ))
                    },
                )?;

                let initial_capital: f64 = self.conn.query_row(
                    "SELECT initial_capital FROM runs WHERE start_ts = ?1 LIMIT 1",
                    params![run_key],
                    |r| r.get(0),
                )?;

                (
                    initial_capital + realized_pnl,
                    realized_pnl,
                    total_trades,
                    win_count,
                    loss_count,
                )
            }
        };

        let last_trade_exit_ts = self.conn.query_row(
            "
            SELECT MAX(exit_ts) FROM trades
            WHERE run_key = ?1 AND exit_ts IS NOT NULL
            ",
            params![run_key],
            |r| r.get::<_, Option<i64>>(0),
        )?;

        let initial_capital: f64 = self.conn.query_row(
            "SELECT initial_capital FROM runs WHERE start_ts = ?1 LIMIT 1",
            params![run_key],
            |r| r.get(0),
        )?;

        Ok(Some(ResumeState {
            run_key,
            position,
            capital,
            realized_pnl,
            total_trades,
            win_count,
            loss_count,
            last_trade_exit_ts,
            initial_capital,
        }))
    }

    fn ensure_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS runs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                start_ts INTEGER NOT NULL,
                end_ts INTEGER,
                coin TEXT NOT NULL,
                initial_capital REAL NOT NULL,
                final_capital REAL,
                config TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS signals (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                run_key INTEGER NOT NULL,
                ts INTEGER NOT NULL,
                coin TEXT NOT NULL,
                price REAL NOT NULL,
                bid REAL NOT NULL,
                ask REAL NOT NULL,
                spread REAL NOT NULL,
                ob_imbalance REAL NOT NULL,
                last_close REAL NOT NULL,
                prev_close REAL NOT NULL,
                momentum TEXT NOT NULL,
                signal TEXT NOT NULL,
                reason TEXT,
                position_open INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS signals_run_key_idx ON signals (run_key);

            CREATE TABLE IF NOT EXISTS trades (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                run_key INTEGER NOT NULL,
                coin TEXT NOT NULL,
                direction TEXT NOT NULL,
                entry_ts INTEGER NOT NULL,
                entry_px REAL NOT NULL,
                size REAL NOT NULL,
                notional REAL NOT NULL,
                exit_ts INTEGER,
                exit_px REAL,
                gross_pnl REAL,
                entry_fee REAL,
                exit_fee REAL,
                net_pnl REAL,
                exit_reason TEXT
            );
            CREATE INDEX IF NOT EXISTS trades_run_key_idx ON trades (run_key);

            CREATE TABLE IF NOT EXISTS equity (
                run_key INTEGER NOT NULL,
                ts INTEGER NOT NULL,
                capital REAL NOT NULL,
                unrealized_pnl REAL NOT NULL,
                realized_pnl REAL NOT NULL,
                total_trades INTEGER NOT NULL,
                win_count INTEGER NOT NULL,
                loss_count INTEGER NOT NULL,
                PRIMARY KEY (run_key, ts)
            );
            CREATE INDEX IF NOT EXISTS equity_run_key_idx ON equity (run_key);
            ",
        )?;
        Ok(())
    }

    fn bump_writes(&mut self) -> Result<()> {
        self.write_count += 1;
        if self.write_count >= CHECKPOINT_EVERY {
            self.conn
                .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
            self.write_count = 0;
        }
        Ok(())
    }
}
