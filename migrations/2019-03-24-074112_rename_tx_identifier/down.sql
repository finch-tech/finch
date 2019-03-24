-- This file should undo anything in `up.sql`
ALTER TABLE btc_transactions RENAME hash TO txid;