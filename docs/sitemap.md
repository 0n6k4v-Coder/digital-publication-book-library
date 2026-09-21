# Sitemap

## Overview

The application is divided into three areas:

- **Public Website** — discover stories, books, and curated collections.
- **Admin Panel** — manage posts and books.
- **System** — authentication and error pages.

---

## Sitemap

```
/
├── Stories
│   └── /stories/:slug
│
├── Library
│   ├── /library
│   ├── /library/category/:slug
│   └── /library/:slug
│
├── Collections
│   ├── /collections
│   └── /collections/:slug
│
├── About
│
└── 404

/admin
├── Login
├── Dashboard
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

# Public Website

## Core Pages

| Page ID | Route | Page | Parent |
|---|---|---|---|
| `PAGE-PUB-001` | `/` | Homepage | — |
| `PAGE-PUB-002` | `/stories` | Stories / News | `/` |
| `PAGE-PUB-003` | `/stories/:slug` | Story Detail | `/stories` |
| `PAGE-PUB-004` | `/library` | Book Library | `/` |
| `PAGE-PUB-005` | `/library/:slug` | Book Detail | `/library` |
| `PAGE-PUB-006` | `/collections` | Collections | `/` |
| `PAGE-PUB-007` | `/collections/:slug` | Collection Detail | `/collections` |
| `PAGE-PUB-008` | `/about` | About | `/` |

## Library Category

| Page ID | Route | Page | Parent |
|---|---|---|---|
| `PAGE-PUB-009` | `/library/category/:slug` | Book Category | `/library` |

---

# Admin Panel

## Authentication

| Page ID | Route | Page | Parent |
|---|---|---|---|
| `PAGE-ADM-AUTH-001` | `/admin/login` | Admin Login | `/admin` |

## Dashboard

| Page ID | Route | Page | Parent |
|---|---|---|---|
| `PAGE-ADM-001` | `/admin` | Dashboard | — |

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

---

# Navigation

## Public Navigation

```
Logo
├── Stories
├── Library
├── Collections
└── About
```

## Admin Navigation

```
Dashboard
├── Posts
└── Books
```

---

# Homepage Relationships

```
Homepage
│
├── Featured Story
│   └── Story Detail
│
├── Latest Stories
│   └── Stories
│       └── Story Detail
│
├── Featured Books
│   └── Book Detail
│
├── Reading Room
│   └── Book Library
│
├── Curated Collection
│   └── Collection Detail
│       └── Story / Book
│
└── Explore Library
    └── Book Library
```

---

# Library Relationships

```
Book Library
│
├── Category
│   └── Book Category
│       └── Book Detail
│
├── Book Card
│   └── Book Detail
│       └── Download PDF
│
└── Collection
    └── Collection Detail
        └── Book Detail
```

---

# Admin Relationships

```
Dashboard
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

---

# Route Convention

| Resource | Route Pattern |
|---|---|
| Homepage | `/` |
| Stories | `/stories` |
| Story detail | `/stories/:slug` |
| Library | `/library` |
| Library category | `/library/category/:slug` |
| Book detail | `/library/:slug` |
| Collections | `/collections` |
| Collection detail | `/collections/:slug` |
| About | `/about` |
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
| Public Website | 9 |
| Admin Panel | 8 |
| System | 1 |
| **Total** | **18** |

Dynamic routes such as `/stories/:slug` and `/library/:slug` use shared page templates rather than separate designs for every item.
