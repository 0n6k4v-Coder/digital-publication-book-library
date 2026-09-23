# Sitemap

## Overview

The application is kept intentionally small and focused on the core brief:

- **Public Website** — browse published posts and books, then open a post or book detail page.
- **Admin Panel** — manage posts, books, and administrator accounts.
- **System** — authentication and error handling.

---

## Sitemap

```
/
├── /posts
│   └── /posts/:slug
│
└── /books
    └── /book/:slug

/login

/admin
├── /posts
│   ├── /new
│   └── /:id/edit
│
├── /books
│   ├── /new
│   └── /:id/edit
│
└── /accounts
    ├── /create
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
| `PAGE-PUB-002` | `/posts` | Posts | `/` |
| `PAGE-PUB-003` | `/posts/:slug` | Post Detail | `/posts` |
| `PAGE-PUB-004` | `/books` | Book Library | `/` |
| `PAGE-PUB-005` | `/book/:slug` | Book Detail | `/books` |

The homepage is the main landing page and may surface the latest posts. The dedicated `/posts` route provides the public post collection.

---

# Admin Panel

## Authentication

| Page ID | Route | Page | Parent |
|---|---|---|---|
| `PAGE-ADM-AUTH-001` | `/login` | Admin Login | — |

The login page is the entry point for authorized administrators. Public administrator registration is not part of the application scope.

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

## Administrator Accounts

| Page ID | Route | Page | Parent |
|---|---|---|---|
| `PAGE-ADM-008` | `/admin/accounts` | Administrator Accounts | `/admin` |
| `PAGE-ADM-009` | `/admin/accounts/create` | Administrator Account — Create | `/admin/accounts` |
| `PAGE-ADM-010` | `/admin/accounts/:id/edit` | Administrator Account — Edit | `/admin/accounts` |

There is no separate dashboard page in this scope. Admin entry can route directly to the content-management area.

---

# Navigation

## Public Navigation

```
Logo
├── Posts
│   └── Posts
└── Books
    └── Book Library
```

## Admin Navigation

```
Admin
├── Posts
├── Books
└── Accounts
    ├── Administrator Accounts
    ├── Create Account
    └── Edit Account
```

---

# Homepage Relationships

```
Homepage
│
├── Latest Posts
│   └── Post Detail
│
├── Featured Books
│   └── Book Detail
│
└── Explore the Library
    └── Book Library
```

---

# Posts Relationships

```
Posts
│
└── Post Card
    └── Post Detail
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
├── Books
│   ├── New Book
│   └── Edit Book
│
└── Accounts
    ├── Administrator Accounts
    ├── Create Account
    └── Edit Account
```

---

# System Pages

| Page ID | Route | Page | Purpose |
|---|---|---|---|
| `PAGE-SYS-001` | `*` | 404 Not Found | Handle invalid routes. |

Admin authentication is required by `ADM-AUTH-001`. Authorized administrators manage other administrator accounts through the protected accounts section.

---

# Route Convention

| Resource | Route Pattern |
|---|---|
| Homepage | `/` |
| Posts | `/posts` |
| Post detail | `/posts/:slug` |
| Library | `/books` |
| Book detail | `/book/:slug` |
| Admin | `/admin` |
| Admin login | `/login` |
| Posts management | `/admin/posts` |
| New post | `/admin/posts/new` |
| Edit post | `/admin/posts/:id/edit` |
| Books management | `/admin/books` |
| New book | `/admin/books/new` |
| Edit book | `/admin/books/:id/edit` |
| Administrator accounts | `/admin/accounts` |
| Create administrator account | `/admin/accounts/create` |
| Edit administrator account | `/admin/accounts/:id/edit` |

---

# Page Count

| Area | Routes |
|---|---:|
| Public Website | 5 |
| Admin Panel | 10 |
| System | 1 |
| **Total** | **16** |

Dynamic routes such as `/posts/:slug` and `/book/:slug` use shared page templates rather than separate designs for every item.
