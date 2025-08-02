# Interactive Learning System for the Rust Fullstack Starter

*Learn this specific starter system through AI-powered interactive teaching*

---

## 🎯 What This Is

This course teaches you to **completely master THIS specific Rust Fullstack Starter**, not general web development. By the end, you'll understand every line of code and be able to create your own custom system using the rename script.

## 🚀 Quick Start Guide

### Step 1: Create Your Personal Student Book
```bash
# Copy the student book template and make it yours
cp docs/courses/STUDENT_BOOK.md docs/courses/STUDENT_BOOK.your-name.md

# Example personalized names:
# STUDENT_BOOK.john.md
# STUDENT_BOOK.sarah.md
# STUDENT_BOOK.alex-dev.md

# The .gitignore will keep your personal learning journal private
# while allowing you to commit other course materials
```

### Step 2: Set Up Your AI Learning Assistant

**Recommended AI assistants with codebase access:**
- **Claude Code** (claude.ai/code) - Direct codebase integration, can read and explore files
- **Gemini CLI** - Command-line AI with live codebase access
- **Cursor** - AI-powered code editor with full project context

**Alternative for web-based AI assistants:**
- Claude (claude.ai) - Upload both course files
- ChatGPT (chat.openai.com) - Upload both course files  
- Any other LLM with file upload capability

**For codebase-integrated AI (Recommended):**
The AI can directly explore files mentioned in lessons and provide real-time guidance based on the actual code.

**For web-based AI:**
Upload these two files to start your interactive learning session:
1. `docs/courses/TEACHER_BOOK.md` - Complete teaching guide
2. `docs/courses/STUDENT_BOOK.your-name.md` - Your personal learning journal

### Step 3: Start Your First Learning Session

**For codebase-integrated AI (Claude Code, Gemini CLI):**
```
I want to learn the Rust Fullstack Starter system completely using the interactive learning system in docs/courses/. 

Please act as my Feynman-method teacher for this specific starter system. Read docs/courses/TEACHER_BOOK.md for the complete curriculum and help me work through it systematically starting with Lesson 1: System Overview.

Teaching rules:
1. Always reference actual files in the codebase (like starter/src/main.rs)
2. Ask me to experiment with real code changes you can verify
3. Use your codebase access to provide specific examples and line numbers
4. Focus only on THIS starter system, not general concepts
5. Guide me to truly understand, not just memorize

Let's begin! What should I do first to understand the system overview?
```

**For web-based AI (after uploading both course files):**
```
I want to learn the Rust Fullstack Starter system completely. I've uploaded the TEACHER_BOOK.md with the complete curriculum and my personal STUDENT_BOOK.your-name.md learning journal.

Please act as my Feynman-method teacher for this specific starter system. I want to start with Lesson 1: System Overview.

Teaching rules:
1. Always reference actual files in the codebase (like starter/src/main.rs)
2. Ask me to experiment with real code changes
3. Help me update my learning journal as we progress
4. Focus only on THIS starter system, not general concepts
5. Guide me to truly understand, not just memorize

Let's begin! What should I do first to understand the system overview?
```

## 📚 Learning Methodology

### The Feynman Method Applied to This Starter

1. **Understand** - We'll explore actual files in the codebase together
2. **Simplify** - Break down complex parts into understandable pieces
3. **Teach Back** - You'll explain concepts back to me in your own words
4. **Iterate** - We'll refine understanding through experiments and questions

### Active Learning Principles

- **No Passive Reading** - Every concept involves hands-on exploration
- **Question Everything** - Why did the authors choose this approach?
- **Experiment Freely** - Modify code to see what happens
- **Connect the Dots** - Understand how all parts work together
- **Build Real Things** - Add actual features to prove understanding

### Your AI Teaching Assistant Will:

✅ **Guide you through the 15-lesson curriculum systematically**
✅ **Reference specific files in the actual codebase**
✅ **Suggest real experiments and code modifications**
✅ **Help you update your learning journal with insights**
✅ **Answer questions specific to this starter system**
✅ **Challenge you with hands-on exercises**

**Additional benefits with codebase-integrated AI:**
✅ **Read and analyze actual code in real-time**
✅ **Provide exact line numbers and code examples**
✅ **Verify your experiments and modifications**
✅ **Navigate the complete project structure**
✅ **Keep up with any codebase changes**

❌ **Won't teach general web development concepts**
❌ **Won't skip the systematic progression**
❌ **Won't let you treat any code as a "magic box"**

---

## 🎓 Course Structure (15 Lessons)

### 📖 Phase 1: Backend Mastery (Lessons 1-8)
Master every Rust file in `starter/src/` before touching frontend

### 🌐 Phase 2: Frontend Integration (Lessons 9-13)
Understand how `web/` connects to the backend you've mastered

### 🔧 Phase 3: Customization & Mastery (Lessons 14-15)
Use the rename script to create your own system

---

## 💡 Sample Learning Session

**You:** "I'm ready for Lesson 2 about the database foundation."

**AI Teacher:** "Great! Let's explore the database layer. First, run this command to see all the migrations:

```bash
ls starter/migrations/
```

You should see 5 migration files. Open `001_users.up.sql` and tell me what you notice about the `users` table structure. What fields are there and why do you think each exists in THIS specific starter?"

**You:** [Experiment and respond]

**AI Teacher:** "Perfect observation! Now let's see how `starter/src/database.rs` connects to these tables. Look at line 25 where the connection pool is configured. What happens if we change the pool size? Let's find out..."

---

## 🔄 Continuous Learning Loop

1. **Learn** - AI teaches you a specific concept from the curriculum
2. **Experiment** - You modify actual code to test understanding
3. **Journal** - Update your learning journal with insights
4. **Question** - Ask follow-up questions about what you discovered
5. **Apply** - Build on the knowledge in the next lesson
6. **Repeat** - Continue until you've mastered the entire system

---

## 🎯 Success Indicators

**You'll know you're succeeding when:**
- You can navigate to any file quickly and understand its purpose
- You can trace a request from frontend to database and back
- You can add new features following existing patterns
- You can debug problems systematically
- You feel confident modifying any part of the system

**Your final success:**
- Successfully using `scripts/rename-project.sh` to create your own custom system
- Adding substantial new features independently
- Teaching someone else how the system works

---

## 🆘 Getting Help

**If you get stuck:**
1. Reference the specific lesson in TEACHER_BOOK.md
2. Ask your AI assistant to explain the concept differently
3. Try a simpler experiment first
4. Look at the actual test files to see usage examples
5. Run the quality checks: `./scripts/check.sh`

**If the AI assistant gets off track:**
- Remind it: "Please focus only on THIS starter system, not general concepts"
- Ask: "Can you show me the specific file in the codebase where this happens?"
- Redirect: "Let's get back to Lesson X in the TEACHER_BOOK.md"

---

## 📊 Track Your Progress

**Update your learning journal regularly:**
- Mark lessons as complete ✅
- Document your "aha!" moments 💡
- Record experiments you tried 🔬
- Note questions for later exploration ❓
- Celebrate breakthroughs 🎉

---

## 🎉 Graduation

**You've mastered the system when you can:**
1. Rename the project successfully using the rename script
2. Add a new entity with full CRUD operations
3. Create a new task type with custom processing
4. Add a new admin dashboard page
5. Teach someone else how any part works

**Then you're ready to build amazing things with your new knowledge!**

---

*"I learned very early the difference between knowing the name of something and knowing something." - Richard P. Feynman*

*Now start your journey to truly KNOW this system inside and out!*