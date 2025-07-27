#!/bin/bash
# Database backup script for production
# Runs inside postgres container

set -e

# Configuration
BACKUP_DIR="/backups"
DB_NAME="${POSTGRES_DB:-starter_prod}"
DB_USER="${POSTGRES_USER:-starter_user}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="${BACKUP_DIR}/backup_${DB_NAME}_${TIMESTAMP}.sql"
KEEP_DAYS=${BACKUP_KEEP_DAYS:-7}

# Ensure backup directory exists
mkdir -p "$BACKUP_DIR"

echo "Starting database backup..."
echo "Database: $DB_NAME"
echo "Backup file: $BACKUP_FILE"

# Create backup
pg_dump -U "$DB_USER" -d "$DB_NAME" --verbose --clean --if-exists --create > "$BACKUP_FILE"

# Compress backup
gzip "$BACKUP_FILE"
BACKUP_FILE="${BACKUP_FILE}.gz"

echo "Backup completed: $BACKUP_FILE"

# Clean up old backups
echo "Cleaning up backups older than $KEEP_DAYS days..."
find "$BACKUP_DIR" -name "backup_${DB_NAME}_*.sql.gz" -type f -mtime +$KEEP_DAYS -delete

# List current backups
echo "Current backups:"
ls -lh "$BACKUP_DIR"/backup_${DB_NAME}_*.sql.gz 2>/dev/null || echo "No backups found"

echo "Backup process completed successfully"