-- Migration 002: Ledger Immutability Triggers

CREATE OR REPLACE FUNCTION raise_immutable_error()
RETURNS TRIGGER AS $$
BEGIN
  RAISE EXCEPTION 'Table % is immutable: % operation not allowed', TG_TABLE_NAME, TG_OP;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER ledger_immutable_trigger
  BEFORE UPDATE OR DELETE ON ledger
  FOR EACH ROW EXECUTE FUNCTION raise_immutable_error();

