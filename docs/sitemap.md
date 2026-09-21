# Sitemap

## Overview

The application is kept intentionally small and focused on the core brief:

- **Public Website** — browse published posts and books, then open a post or book detail page.
- **Admin Panel** — manage posts and books.
- **System** — authentication and error handling.

---

## Sitemap

```
/
├── /stories/:slug
│
└── /books
    └── /book/:slug

/admin
├── /login
├── /posts
│   ├── /new
│   └── /:id/edit
│
└── /books
    ├── /new
    └── /:id/edit

*
└── 404
```

---

# Public Website

## Core Pages

| Page ID | Route | Page | Parent |
|---|---|---|---|
| `PAGE-PUB-001` | `/` | Homepage | — |
| `PAGE-PUB-003` | `/stories/:slug` | Story Detail | `/` |
| `PAGE-PUB-004` | `/books` | Book Library | `/` |
| `PAGE-PUB-005` | `/book/:slug` | Book Detail | `/books` |

The homepage is the main public post surface. There is no separate stories listing route in this scope.

---

# Admin Panel

## Authentication

| Page ID | Route | Page | Parent |
|---|---|---|---|
| `PAGE-ADM-AUTH-001` | `/admin/login` | Admin Login | `/admin` |

## Posts

| Page ID | Route | Page | Parent |
|---|---|---|---|
| `PAGE-ADM-002` | `/admin/posts` | Posts Management | `/admin` |
| `PAGE-ADM-003` | `/admin/posts/new` | Post Editor — Create | `/admin/posts` |
| `PAGE-ADM-004` | `/admin/posts/:id/edit` | Post Editor — Edit | `/admin/posts` |

## Books

| Page ID | Route | Page | Parent |
|---|---|---|---|
| `PAGE-ADM-005` | `/admin/books` | Books Management | `/admin` |
| `PAGE-ADM-006` | `/admin/books/new` | Book Editor — Create | `/admin/books` |
| `PAGE-ADM-007` | `/admin/books/:id/edit` | Book Editor — Edit | `/admin/books` |

There is no separate dashboard page in this scope. Admin entry can route directly to the content-management area.

---

# Navigation

## Public Navigation

```
Logo
├── Stories
│   └── Homepage latest-posts section
└── Library
    └── Book Library
```

## Admin Navigation

```
Admin
├── Posts
└── Books
```

---

# Homepage Relationships

```
Homepage
│
├── Latest Posts
│   └── Story Detail
│
├── Featured Books
│   └── Book Detail
│
└── Explore the Library
    └── Book Library
```

---

# Library Relationships

```
Book Library
│
└── Book Card
    └── Book Detail
        └── Download PDF
```

---

# Admin Relationships

```
Admin
│
├── Posts
│   ├── New Post
│   └── Edit Post
│
└── Books
    ├── New Book
    └── Edit Book
```

---

# System Pages

| Page ID | Route | Page | Purpose |
|---|---|---|---|
| `PAGE-SYS-001` | `*` | 404 Not Found | Handle invalid routes. |

Admin authentication is required by `ADM-AUTH-001` but does not need a separate sitemap page beyond the login route.

---

# Route Convention

| Resource | Route Pattern |
|---|---|
| Homepage | `/` |
| Story detail | `/stories/:slug` |
| Library | `/books` |
| Book detail | `/book/:slug` |
| Admin | `/admin` |
| Admin login | `/admin/login` |
| Posts | `/admin/posts` |
| New post | `/admin/posts/new` |
| Edit post | `/admin/posts/:id/edit` |
| Books | `/admin/books` |
| New book | `/admin/books/new` |
| Edit book | `/admin/books/:id/edit` |

---

# Page Count

| Area | Routes |
|---|---:|
| Public Website | 4 |
| Admin Panel | 7 |
| System | 1 |
| **Total** | **12** |

Dynamic routes such as `/stories/:slug` and `/book/:slug` use shared page templates rather than separate designs for every item.
