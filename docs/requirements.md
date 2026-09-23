# Digital Publication & Book Library Web Application

> **Project Type:** Self-initiated prototype  
> **Source:** Real-world project brief found on Fastwork  
> **Purpose:** Personal challenge and portfolio project  
> **Status:** Prototype — not a commissioned project

---

## Original Brief from [Fastwork](https://jobboard.fastwork.co/jobs/%E0%B8%9E%E0%B8%B1%E0%B8%92%E0%B8%99%E0%B8%B2%E0%B9%80%E0%B8%A7%E0%B9%87%E0%B8%9A%E0%B9%84%E0%B8%8B%E0%B8%95%E0%B9%8C/9ae4f57c-d5f0-474b-8d9d-7f663a95ea0f?source=web_jobboard_job-listing)

> ฟังก์ชั่นง่ายๆครับ
>
> 1. ขอแค่สามารถลงไฟล์ PDF ไฟล์หนังสือให้คนดาวน์โหลดได้
> 2. มีหน้าแรกไว้ลงโพสต์ต่างๆ
>
> ขอแค่ 2 ฟังก์ชั่นง่ายๆครับคิดว่าไม่ยาก  
> ขอคนออกแบบเว็ปที่สวยงาม

### Brief Translation

The website requires two main functions:

1. Upload PDF book files and allow visitors to download them.
2. Provide a homepage for publishing posts.

The website should be simple and visually appealing.

---

## Table of Contents

- [Project Overview](#project-overview)
- [Goals](#goals)
- [Requirement ID Convention](#requirement-id-convention)
- [Functional Requirements](#functional-requirements)
  - [Public Website](#public-website)
  - [Admin Panel](#admin-panel)
- [Content Requirements](#content-requirements)
- [Scope](#scope)
- [Out of Scope](#out-of-scope)
- [Traceability](#traceability)

---

## Project Overview

A simple web application combining:

- **Digital publication** — publish and display posts.
- **Book library** — publish PDF books and allow visitors to download them.

The application will be developed as a self-initiated prototype based on the original brief.

---

## Goals

| ID | Description |
|---|---|
| `G-001` | Provide a visually polished and usable website. |
| `G-002` | Allow administrators to manage published posts. |
| `G-003` | Allow administrators to manage PDF books. |
| `G-004` | Allow visitors to browse published content. |
| `G-005` | Allow visitors to download published PDF books. |
| `G-006` | Keep the system simple and focused on the original brief. |

---

# Requirement ID Convention

Requirement IDs use the following format:

```
<AREA>-<FEATURE>-<NUMBER>
```

| Prefix | Area |
|---|---|
| `PUB` | Public website |
| `ADM` | Admin panel |
| `SYS` | System |
| `CNT` | Content/data |
| `G` | Goals |
| `SCP` | Scope |
| `OUT` | Out of scope |

Examples:

```
PUB-HOME-001
PUB-POST-001
PUB-BOOK-001
ADM-POST-001
ADM-BOOK-001
```

Requirement IDs are stable identifiers and should be referenced in:

- Design files
- Development tasks
- GitHub issues
- Pull requests
- Testing documentation

---

# Functional Requirements

## Public Website

### Homepage

| ID | Requirement | Priority |
|---|---|---|
| `PUB-HOME-001` | The homepage must display published posts. | Must |
| `PUB-HOME-002` | The homepage must provide navigation to the book library. | Must |
| `PUB-HOME-003` | The homepage may display featured or latest books. | Should |

### Posts

| ID | Requirement | Priority |
|---|---|---|
| `PUB-POST-001` | Visitors must be able to view published posts. | Must |
| `PUB-POST-002` | Visitors must be able to open an individual post. | Must |
| `PUB-POST-003` | A post detail page must display the post title and content. | Must |
| `PUB-POST-004` | A post may display a featured image and publication date. | Should |

### Book Library

| ID | Requirement | Priority |
|---|---|---|
| `PUB-BOOK-001` | Visitors must be able to browse published books. | Must |
| `PUB-BOOK-002` | Each book must display its cover, title, and basic information. | Must |
| `PUB-BOOK-003` | Visitors must be able to open an individual book. | Must |
| `PUB-BOOK-004` | The book detail page must display the book information. | Must |
| `PUB-BOOK-005` | Visitors must be able to download the published PDF file. | Must |

---

# Admin Panel

## Admin Access

| ID | Requirement | Priority |
|---|---|---|
| `ADM-AUTH-001` | The admin area must be restricted to authorized administrators. | Must |
| `ADM-AUTH-002` | The system must support multiple authorized administrator accounts. | Must |
| `ADM-AUTH-003` | Authorized administrators must be able to create additional administrator accounts. | Must |
| `ADM-AUTH-004` | Authorized administrators must be able to view administrator accounts. | Must |
| `ADM-AUTH-005` | Authorized administrators must be able to edit administrator accounts. | Must |
| `ADM-AUTH-006` | Authorized administrators must be able to deactivate administrator accounts. | Must |
| `ADM-AUTH-007` | The system must prevent deactivation of the last active administrator account. | Must |

Administrator accounts are managed inside the protected admin area. There is no public administrator registration flow. All administrator accounts have the same access level in this scope; role-based permissions are not required.

## Post Management

| ID | Requirement | Priority |
|---|---|---|
| `ADM-POST-001` | Admins must be able to view all posts. | Must |
| `ADM-POST-002` | Admins must be able to create a post. | Must |
| `ADM-POST-003` | Admins must be able to edit a post. | Must |
| `ADM-POST-004` | Admins must be able to delete a post. | Must |
| `ADM-POST-005` | Admins must be able to publish or unpublish a post. | Must |

## Book Management

| ID | Requirement | Priority |
|---|---|---|
| `ADM-BOOK-001` | Admins must be able to view all books. | Must |
| `ADM-BOOK-002` | Admins must be able to create a book. | Must |
| `ADM-BOOK-003` | Admins must be able to edit a book. | Must |
| `ADM-BOOK-004` | Admins must be able to delete a book. | Must |
| `ADM-BOOK-005` | Admins must be able to upload a PDF file for a book. | Must |
| `ADM-BOOK-006` | Admins must be able to replace an existing PDF file. | Must |
| `ADM-BOOK-007` | Admins must be able to publish or unpublish a book. | Must |

---

# Content Requirements

## Post

| ID | Field | Required |
|---|---|---|
| `CNT-POST-001` | Title | Yes |
| `CNT-POST-002` | Content | Yes |
| `CNT-POST-003` | Featured Image | No |
| `CNT-POST-004` | Publication Date | No |
| `CNT-POST-005` | Status | Yes |

## Book

| ID | Field | Required |
|---|---|---|
| `CNT-BOOK-001` | Title | Yes |
| `CNT-BOOK-002` | Author | No |
| `CNT-BOOK-003` | Cover Image | No |
| `CNT-BOOK-004` | Description | No |
| `CNT-BOOK-005` | Category | No |
| `CNT-BOOK-006` | PDF File | Yes |
| `CNT-BOOK-007` | Status | Yes |

## Administrator Account

| ID | Field | Required |
|---|---|---|
| `CNT-ADMIN-001` | Name | Yes |
| `CNT-ADMIN-002` | Email | Yes |
| `CNT-ADMIN-003` | Password Credential | Yes |
| `CNT-ADMIN-004` | Status | Yes |

---

# Scope

| ID | Description |
|---|---|
| `SCP-001` | Public homepage for displaying published posts. |
| `SCP-002` | Public post listing and post detail pages. |
| `SCP-003` | Public book library and book detail pages. |
| `SCP-004` | PDF book download functionality. |
| `SCP-005` | Admin authentication and access control. |
| `SCP-006` | Admin post management. |
| `SCP-007` | Admin book and PDF management. |
| `SCP-008` | Post and book publishing status management. |
| `SCP-009` | Administrator account management, including viewing, creating, editing, and deactivating administrator accounts. |

---

# Out of Scope

| ID | Description |
|---|---|
| `OUT-001` | Visitor registration and login. |
| `OUT-002` | Online payment. |
| `OUT-003` | E-commerce functionality. |
| `OUT-004` | Book borrowing or lending. |
| `OUT-005` | Online PDF editing. |
| `OUT-006` | User comments. |
| `OUT-007` | Ratings and reviews. |
| `OUT-008` | Social features. |
| `OUT-009` | Complex library management. |
| `OUT-010` | Membership system. |
| `OUT-011` | Advanced recommendation system. |
| `OUT-012` | Complex search and filtering. |
| `OUT-013` | Role-based administrator permissions or a multi-level authorization system. |
| `OUT-014` | Public administrator registration or self-service administrator signup. |

---

# Traceability

Every design and development task should reference one or more requirement IDs.

| Area | Example IDs |
|---|---|
| Figma — Homepage | `PUB-HOME-001`, `PUB-HOME-002` |
| Figma — Book Detail | `PUB-BOOK-003`, `PUB-BOOK-004`, `PUB-BOOK-005` |
| Frontend — Posts | `PUB-POST-001`, `PUB-POST-002` |
| Frontend — Books | `PUB-BOOK-001`, `PUB-BOOK-005` |
| Admin — Posts | `ADM-POST-002`, `ADM-POST-003`, `ADM-POST-005` |
| Admin — Books | `ADM-BOOK-002`, `ADM-BOOK-005`, `ADM-BOOK-007` |
| Admin — Accounts | `ADM-AUTH-002`, `ADM-AUTH-003`, `ADM-AUTH-004`, `ADM-AUTH-005`, `ADM-AUTH-006`, `ADM-AUTH-007` |
| Admin — Login | `ADM-AUTH-001`, `ADM-AUTH-002` |
| Testing | Reference the specific requirement being tested |

### Traceability Flow

```
Requirement
    ↓
Design
    ↓
Implementation
    ↓
Test
```

Example:

```
PUB-BOOK-005
    ↓
Book Detail — Download Button
    ↓
PDF Download Implementation
    ↓
Verify PDF can be downloaded
```

The requirement ID should remain consistent throughout the project so that each feature can be traced from the original requirement to its design, implementation, and test.
