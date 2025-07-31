#!/bin/bash
# rename-project.sh - Rename the starter project to your custom name
# Usage: ./scripts/rename-project.sh <new-project-name>

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if new name is provided
if [ $# -eq 0 ]; then
    echo -e "${RED}❌ Error: Project name is required${NC}"
    echo ""
    echo "Usage: $0 <new-project-name>"
    echo "Example: $0 my_awesome_project"
    echo ""
    echo "Requirements:"
    echo "- Name must contain only letters, numbers, and underscores"
    echo "- Name must start with a letter or underscore"
    echo "- Name should be in snake_case for Rust conventions"
    exit 1
fi

NEW_NAME="$1"
NEW_NAME_UPPER=$(echo "$NEW_NAME" | tr '[:lower:]' '[:upper:]')

# Validate project name (Rust package naming conventions)
if [[ ! "$NEW_NAME" =~ ^[a-zA-Z_][a-zA-Z0-9_]*$ ]]; then
    echo -e "${RED}❌ Error: Invalid project name format${NC}"
    echo ""
    echo "Project name must:"
    echo "- Start with a letter or underscore"
    echo "- Contain only letters, numbers, and underscores"
    echo "- Follow Rust package naming conventions"
    echo ""
    echo "Good examples: my_project, awesome_app, backend_service"
    echo "Bad examples: 123project, my-project, project.name"
    exit 1
fi

# Check if already renamed
if [ ! -d "starter" ]; then
    echo -e "${YELLOW}⚠️  Warning: 'starter' directory not found${NC}"
    echo "This project may have already been renamed, or you're in the wrong directory."
    echo ""
    echo "Current directory: $(pwd)"
    echo "Expected to find: starter/"
    echo ""
    read -p "Continue anyway? (y/N): " confirm
    if [[ ! $confirm =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 1
    fi
else
    # Check if target directory already exists
    if [ -d "$NEW_NAME" ]; then
        echo -e "${RED}❌ Error: Directory '$NEW_NAME' already exists${NC}"
        echo "Please choose a different name or remove the existing directory."
        exit 1
    fi
fi

echo -e "${BLUE}🚀 Renaming project from 'starter' to '$NEW_NAME'...${NC}"
echo ""

# 0. Stop any running Docker services (environment variables will change)
echo -e "${BLUE}🐳 Stopping Docker services (environment will change)...${NC}"
if command -v docker-compose >/dev/null 2>&1; then
    docker-compose down --remove-orphans 2>/dev/null || true
elif command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
    docker compose down --remove-orphans 2>/dev/null || true
else
    echo -e "${YELLOW}⚠️  Docker Compose not found, skipping container shutdown${NC}"
fi

# Create backup timestamp
BACKUP_TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="backup_${BACKUP_TIMESTAMP}"

echo -e "${YELLOW}📦 Creating backup in $BACKUP_DIR/${NC}"
mkdir -p "$BACKUP_DIR"
if [ -d "starter" ]; then
    cp -r starter/ "$BACKUP_DIR/"
fi
cp Cargo.toml "$BACKUP_DIR/" 2>/dev/null || true

# 1. Rename the main directory
if [ -d "starter" ]; then
    echo -e "${BLUE}📁 Renaming starter/ directory to $NEW_NAME/${NC}"
    mv starter/ "$NEW_NAME"/
fi

# 2. Update workspace member in root Cargo.toml
echo -e "${BLUE}📝 Updating root Cargo.toml workspace members${NC}"
if [ -f "Cargo.toml" ]; then
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS version
        sed -i '' "s/members = \[\"starter\"\]/members = [\"$NEW_NAME\"]/" Cargo.toml
    else
        # Linux version
        sed -i "s/members = \[\"starter\"\]/members = [\"$NEW_NAME\"]/" Cargo.toml
    fi
fi

# 3. Update package name in binary Cargo.toml
if [ -f "$NEW_NAME/Cargo.toml" ]; then
    echo -e "${BLUE}📝 Updating $NEW_NAME/Cargo.toml package name${NC}"
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS version
        sed -i '' "s/name = \"starter\"/name = \"$NEW_NAME\"/" "$NEW_NAME/Cargo.toml"
    else
        # Linux version
        sed -i "s/name = \"starter\"/name = \"$NEW_NAME\"/" "$NEW_NAME/Cargo.toml"
    fi
fi

# 4. Replace all occurrences of 'starter' with new name in source files
echo -e "${BLUE}🔄 Replacing 'starter' with '$NEW_NAME' in source files...${NC}"

# Find and replace in common file types, excluding certain directories
find . -type f \( -name "*.rs" -o -name "*.toml" -o -name "*.md" -o -name "*.yaml" -o -name "*.yml" -o -name "*.dockerfile" -o -name "Dockerfile" -o -name "*.sh" \) \
    -not -path "./target/*" \
    -not -path "./.git/*" \
    -not -path "./backup_*/*" \
    -not -path "./$NEW_NAME/target/*" \
    -exec grep -l "starter" {} \; | while read -r file; do
    
    echo "  Updating: $file"
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS version - be more careful with replacements
        sed -i '' "s/cargo run --bin starter/cargo run --bin $NEW_NAME/g" "$file"
        sed -i '' "s/--manifest-path starter\//--manifest-path ${NEW_NAME}\//g" "$file"
        sed -i '' "s/cd starter$/cd $NEW_NAME/g" "$file"
        sed -i '' "s/use starter::/use ${NEW_NAME}::/g" "$file"
        sed -i '' "s/use starter;/use ${NEW_NAME};/g" "$file"
        sed -i '' "s/use starter{/use ${NEW_NAME}{/g" "$file"
        sed -i '' "s/starter::/${NEW_NAME}::/g" "$file"
        sed -i '' "s/starter_/${NEW_NAME}_/g" "$file"
        sed -i '' "s/\"starter\"/\"$NEW_NAME\"/g" "$file"
        sed -i '' "s/'starter'/'$NEW_NAME'/g" "$file"
        sed -i '' "s/starter binary/$NEW_NAME binary/g" "$file"
        sed -i '' "s/starter server/$NEW_NAME server/g" "$file"
        sed -i '' "s/starter worker/$NEW_NAME worker/g" "$file"
    else
        # Linux version
        sed -i "s/cargo run --bin starter/cargo run --bin $NEW_NAME/g" "$file"
        sed -i "s/--manifest-path starter\//--manifest-path ${NEW_NAME}\//g" "$file"
        sed -i "s/cd starter$/cd $NEW_NAME/g" "$file"
        sed -i "s/use starter::/use ${NEW_NAME}::/g" "$file"
        sed -i "s/use starter;/use ${NEW_NAME};/g" "$file"
        sed -i "s/use starter{/use ${NEW_NAME}{/g" "$file"
        sed -i "s/starter::/${NEW_NAME}::/g" "$file"
        sed -i "s/starter_/${NEW_NAME}_/g" "$file"
        sed -i "s/\"starter\"/\"$NEW_NAME\"/g" "$file"
        sed -i "s/'starter'/'$NEW_NAME'/g" "$file"
        sed -i "s/starter binary/$NEW_NAME binary/g" "$file"
        sed -i "s/starter server/$NEW_NAME server/g" "$file"
        sed -i "s/starter worker/$NEW_NAME worker/g" "$file"
    fi
done

# 5. Update environment variable prefixes and config
echo -e "${BLUE}🔄 Updating environment variable prefixes...${NC}"
find . -type f \( -name "*.rs" -o -name "*.toml" -o -name "*.md" -o -name "*.yaml" -o -name "*.yml" -o -name "*.env*" -o -name "*.sh" \) \
    -not -path "./target/*" \
    -not -path "./.git/*" \
    -not -path "./backup_*/*" \
    -not -path "./$NEW_NAME/target/*" \
    -exec grep -l "STARTER" {} \; | while read -r file; do
    
    echo "  Updating env vars in: $file"
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS version - Update environment variable prefixes
        sed -i '' "s/with_prefix(\"STARTER\")/with_prefix(\"$NEW_NAME_UPPER\")/g" "$file"
        sed -i '' "s/STARTER__/$NEW_NAME_UPPER\__/g" "$file"
        sed -i '' "s/STARTER_/$NEW_NAME_UPPER\_/g" "$file"
        # Update default database values
        sed -i '' "s/starter_user/${NEW_NAME}_user/g" "$file"
        sed -i '' "s/starter_pass/${NEW_NAME}_pass/g" "$file"
        sed -i '' "s/starter_db/${NEW_NAME}_db/g" "$file"
    else
        # Linux version
        sed -i "s/with_prefix(\"STARTER\")/with_prefix(\"$NEW_NAME_UPPER\")/g" "$file"
        sed -i "s/STARTER__/$NEW_NAME_UPPER\__/g" "$file"
        sed -i "s/STARTER_/$NEW_NAME_UPPER\_/g" "$file"
        # Update default database values
        sed -i "s/starter_user/${NEW_NAME}_user/g" "$file"
        sed -i "s/starter_pass/${NEW_NAME}_pass/g" "$file"
        sed -i "s/starter_db/${NEW_NAME}_db/g" "$file"
    fi
done

# 6. Update script references
echo -e "${BLUE}🔄 Updating script references...${NC}"
if [ -f "scripts/server.sh" ]; then
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/PROJECT_NAME=\"starter\"/PROJECT_NAME=\"$NEW_NAME\"/" scripts/server.sh
    else
        sed -i "s/PROJECT_NAME=\"starter\"/PROJECT_NAME=\"$NEW_NAME\"/" scripts/server.sh
    fi
fi

# 7. Update log file references
echo -e "${BLUE}🔄 Updating log file references...${NC}"
for script in scripts/*.sh; do
    if [ -f "$script" ]; then
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' "s/starter-server/\${NEW_NAME}-server/g" "$script"
            sed -i '' "s/starter-worker/\${NEW_NAME}-worker/g" "$script"
        else
            sed -i "s/starter-server/\${NEW_NAME}-server/g" "$script"
            sed -i "s/starter-worker/\${NEW_NAME}-worker/g" "$script"
        fi
    fi
done

# 8. Restart Docker services with updated environment
echo -e "${BLUE}🐳 Starting Docker services with updated environment...${NC}"
DOCKER_START_SUCCESS=false

# Method 1: Try docker-compose first
if command -v docker-compose >/dev/null 2>&1; then
    if docker-compose up -d 2>/dev/null; then
        echo -e "${GREEN}✅ Docker services started successfully with docker-compose${NC}"
        DOCKER_START_SUCCESS=true
    fi
fi

# Method 2: Try docker compose if docker-compose failed or is not available
if [ "$DOCKER_START_SUCCESS" = false ] && command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
    if docker compose up -d 2>/dev/null; then
        echo -e "${GREEN}✅ Docker services started successfully with docker compose${NC}"
        DOCKER_START_SUCCESS=true
    fi
fi

# Method 3: Force docker compose with verbose output if previous methods failed
if [ "$DOCKER_START_SUCCESS" = false ] && command -v docker >/dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  Previous methods failed, forcing docker compose with verbose output...${NC}"
    if docker compose up -d; then
        echo -e "${GREEN}✅ Docker services started successfully with forced docker compose${NC}"
        DOCKER_START_SUCCESS=true
    else
        echo -e "${RED}❌ All Docker Compose methods failed${NC}"
        echo -e "${YELLOW}💡 Manual fix required: Run 'docker compose up -d' in project root${NC}"
    fi
fi

# Final fallback message
if [ "$DOCKER_START_SUCCESS" = false ]; then
    echo -e "${YELLOW}⚠️  Docker Compose not available or all methods failed${NC}"
    echo -e "${YELLOW}💡 Please manually run 'docker compose up -d' in the project root directory${NC}"
    exit 1
fi

# Step 1: Reset database and run migrations
echo -e "${BLUE}🗄️  Step 1: Reset database and run migrations...${NC}"
if ./scripts/reset-all.sh --reset-database >/dev/null 2>&1; then
    echo -e "${GREEN}✅ Database reset and migrations completed${NC}"
else
    echo -e "${RED}❌ Database reset failed${NC}"
    exit 1
fi

# Step 2: Compile project
echo -e "${BLUE}🔧 Step 2: Compiling project...${NC}"
if cargo check --manifest-path "$NEW_NAME/Cargo.toml" >/dev/null 2>&1; then
    echo -e "${GREEN}✅ Project compiled successfully${NC}"
else
    echo -e "${RED}❌ Compilation failed${NC}"
    exit 1
fi

# 9. Verification
echo ""
echo -e "${GREEN}✅ Renaming complete!${NC}"
echo ""
echo -e "${YELLOW}📋 Summary of changes:${NC}"
echo "  • Stopped Docker services before environment changes"
echo "  • Renamed starter/ → $NEW_NAME/"
echo "  • Updated Cargo.toml workspace members"
echo "  • Updated package name in $NEW_NAME/Cargo.toml"
echo "  • Replaced references in source files"
echo "  • Updated environment variable prefixes (STARTER → $NEW_NAME_UPPER)"
echo "  • Updated config.rs with_prefix to use $NEW_NAME_UPPER"
echo "  • Updated default database values (starter_* → ${NEW_NAME}_*)"
echo "  • Updated script configurations"
echo "  • Restarted Docker services with new environment"
echo "  • Reset database and ran migrations"
echo "  • Compiled project successfully"
echo "  • Created backup in $BACKUP_DIR/"
echo ""
echo -e "${GREEN}🎉 Your project '$NEW_NAME' is ready!${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Test the renamed project:"
echo -e "   ${BLUE}cargo run --bin $NEW_NAME -- --help${NC}"
echo ""
echo "2. Start development server:"
echo -e "   ${BLUE}./scripts/server.sh 3000${NC}"
echo ""
echo "3. Run tests:"
echo -e "   ${BLUE}cargo nextest run${NC}"
echo ""
echo "4. Update README.md with your project description"
echo ""
echo "5. Initialize git repository (if not already done):"
echo -e "   ${BLUE}git add .${NC}"
echo -e "   ${BLUE}git commit -m 'Initial project setup for $NEW_NAME'${NC}"
echo ""
echo -e "${GREEN}Happy coding! 🦀${NC}"