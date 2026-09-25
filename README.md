# Digital Publication & Book Library

A self-initiated prototype based on a real-world website brief found on [Fastwork](https://jobboard.fastwork.co/jobs/%E0%B8%9E%E0%B8%B1%E0%B8%92%E0%B8%99%E0%B8%B2%E0%B9%80%E0%B8%A7%E0%B9%87%E0%B8%9A%E0%B9%84%E0%B8%8B%E0%B8%95%E0%B9%8C/9ae4f57c-d5f0-474b-8d9d-7f663a95ea0f?source=web_jobboard_job-listing). Built as a personal challenge to explore the design and development of a digital publication and book library web application, and to develop a project for my portfolio.

# Docker Command

## Docker Dev Profile Command

### Build

```bash
cp .env.example .env
docker compose -f docker-compose.dev.yml up --build --watch
```

### Clean

```bash
docker compose -f docker-compose.dev.yml down

docker compose -f docker-compose.dev.yml down --volumes --remove-orphans
```