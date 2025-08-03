-- Drop monitoring and observability tables

-- Drop triggers first (function stays since it's from migration 001)
DROP TRIGGER IF EXISTS update_incidents_updated_at ON incidents;
DROP TRIGGER IF EXISTS update_alerts_updated_at ON alerts;

-- Drop tables in reverse dependency order
DROP TABLE IF EXISTS incidents;
DROP TABLE IF EXISTS alerts;
DROP TABLE IF EXISTS metrics;
DROP TABLE IF EXISTS events;