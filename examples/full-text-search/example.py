"""
OverDrive-DB — Full-Text Search Example (Python)
"""
from overdrive import OverDrive

db = OverDrive.open("search_example.odb")

# Insert documents with text content
docs = [
    {"title": "Introduction to Python", "body": "Python is a versatile programming language"},
    {"title": "JavaScript Guide", "body": "JavaScript powers the modern web with dynamic features"},
    {"title": "Rust Performance", "body": "Rust provides memory safety without garbage collection"},
    {"title": "Python Web Frameworks", "body": "Django and Flask are popular Python web frameworks"},
    {"title": "Database Design", "body": "Good database design is essential for scalable applications"},
]

for doc in docs:
    db.insert("articles", doc)

print(f"Inserted {db.count('articles')} articles")

# Full-text search
print("\n--- Search: 'Python' ---")
results = db.search("articles", "Python")
for r in results:
    print(f"  {r.get('title', 'N/A')}")

print("\n--- Search: 'web' ---")
results = db.search("articles", "web")
for r in results:
    print(f"  {r.get('title', 'N/A')}")

print("\n--- Search: 'database' ---")
results = db.search("articles", "database")
for r in results:
    print(f"  {r.get('title', 'N/A')}")

db.close()
print("\nDone!")
