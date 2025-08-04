# Docker Hub Setup Guide

This guide explains how to enable automatic Docker image publishing to Docker Hub for releases.

## Quick Setup

### 1. Create Docker Hub Account

1. Sign up at [hub.docker.com](https://hub.docker.com)
2. Create a new repository (e.g., `your-username/rust-fullstack-starter`)
3. Generate an access token:
   - Go to Account Settings > Security
   - Click "New Access Token"
   - Give it a name like "GitHub Actions"
   - Copy the token (you won't see it again!)

### 2. Configure GitHub Repository Secrets

1. Go to your GitHub repository
2. Navigate to Settings > Secrets and variables > Actions
3. Add these repository secrets:
   - `DOCKER_USERNAME`: Your Docker Hub username
   - `DOCKER_PASSWORD`: Your Docker Hub access token (not your password!)

### 3. Enable Docker Publishing

Edit `.github/workflows/release.yml`:

1. **Update environment variables** (top of file):
   ```yaml
   env:
     CARGO_TERM_COLOR: always
     REGISTRY: docker.io
     IMAGE_NAME: your-dockerhub-username/rust-fullstack-starter
   ```

2. **Uncomment the build-release job** (around line 165):
   - Remove the `#` from all lines in the `build-release` job
   
3. **Update create-release dependencies**:
   ```yaml
   needs: [validate-tag, test, security, build-release]
   ```

### 4. Test the Setup

1. Create a git tag: `git tag v0.1.0`
2. Push the tag: `git push origin v0.1.0`
3. Check GitHub Actions for the release workflow
4. Verify the image appears on Docker Hub

## Example Configuration

After setup, your release workflow will publish multi-platform images:

- `your-username/rust-fullstack-starter:latest`
- `your-username/rust-fullstack-starter:0.1.0`
- `your-username/rust-fullstack-starter:0.1`
- `your-username/rust-fullstack-starter:0`

## Using Published Images

Once published, users can run your application with:

```bash
# Run the latest version
docker run -p 8080:8080 your-username/rust-fullstack-starter:latest

# Run with Docker Compose
echo "services:
  app:
    image: your-username/rust-fullstack-starter:latest
    ports:
      - 8080:8080
    environment:
      - STARTER__DATABASE__HOST=postgres
" > docker-compose.yml

docker-compose up
```

## Security Notes

- **Never commit Docker Hub credentials** to your repository
- **Use access tokens**, not your Docker Hub password
- **Regularly rotate access tokens** for security
- **Limit token permissions** to only push access if possible

## Troubleshooting

### Build Fails with "authentication failed"
- Verify `DOCKER_USERNAME` and `DOCKER_PASSWORD` secrets are set correctly
- Check that the access token has push permissions
- Ensure the repository name matches your Docker Hub repository

### Image not found after push
- Check the image name in the workflow matches your Docker Hub repository
- Verify the repository is public (or you have pull access)
- Images may take a few minutes to appear after push

### Multi-platform build issues
- Remove `platforms: linux/amd64,linux/arm64` if you only need one platform
- Multi-platform builds require more resources and time

## Optional: Deployment Integration

If you enable Docker publishing, you can also enable the deployment job to automatically deploy releases:

1. Uncomment the `deploy-production` job
2. Configure your deployment commands (kubectl, docker-compose, etc.)
3. Add any necessary deployment secrets

This creates a complete CI/CD pipeline: code → test → build → release → deploy.