# Development Scripts

This directory contains helpful scripts for developing and testing the starter project.

## Available Scripts

### Server Management

- **`server.sh`** - Start the server in background with logging
- **`dev-server.sh`** - Start server in development mode with hot reload
- **`dev.sh`** - Development mode with cargo watch
- **`stop-server.sh <port>`** - Stop server running on specified port
- **`test-server.sh <port>`** - Basic health check for server

### Testing

- **`test_auth.sh`** - Test authentication endpoints and session management

## Usage Examples

```bash
# Start development server
./scripts/dev.sh

# Start server in background
./scripts/server.sh

# Test authentication flow
./scripts/test_auth.sh

# Stop server on port 3000
./scripts/stop-server.sh 3000
```

## Notes

- Scripts assume PostgreSQL is running locally
- Default server port is 3000
- Make sure to set up your `.env` file before running
- These are development tools - adapt for your production needs

## Adding New Scripts

When adding new scripts:
1. Make them executable: `chmod +x script_name.sh`
2. Include usage examples in comments
3. Use consistent error handling
4. Update this README