-- Add down migration script here
DROP TABLE IF EXISTS otp_codes;
DROP TRIGGER IF EXISTS trigger_update_timestamp_otp_codes ON otp_codes;
ALTER TABLE users DROP COLUMN is_2fa_enabled,
  DROP COLUMN otp_secret;