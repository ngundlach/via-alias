# Via-Alias

[![build](https://github.com/ngundlach/via-alias/actions/workflows/ci.yml/badge.svg)](https://github.com/ngundlach/via-alias/actions/workflows/ci.yml)

**Via-Alias** is a self-hosted URL shortener built with Rust. It allows you to create short, memorable aliases for long URLs and manage them through a REST API. Built with Axum and SQLite.

Via-Alias is very early stage and under active development. **Expect breaking changes frequently!**

---

## Deployment

You can use the provided Dockerfile to build a containerimage:

```shell
docker build -t via-alias .
```

---

## Via-Alias API Documentation

---

## Redirects

### Get All Redirects

```
GET /api/redirects
```

Returns a list of all registered redirects.

**Response `200 OK`**

```json
{
  "redirects": [
    {
      "alias": "gh",
      "url": "https://github.com"
    }
  ]
}
```

---

### Create Redirect

```
POST /api/redirects
```

**Request Body**

```json
{
  "alias": "gh",
  "url": "https://github.com"
}
```

**Constraints**

- `alias`: 1-50 characters, alphanumeric and hyphens only
- `url`: 1-2048 characters, must start with `http://` or `https://`, no whitespace

**Responses**

- `201 Created` — redirect created, returns the created object
- `400 Bad Request` — invalid input
- `500 Internal Server Error`

**`400` Response Body**

```json
{
  "on_item": "alias",
  "errors": ["can not be empty"]
}
```

---

### Update Redirect

```
PATCH /api/redirects/{alias}
```

**Path Parameters**

| Parameter | Description         |
| --------- | ------------------- |
| `alias`   | The alias to update |

**Request Body**

```json
{
  "url": "https://newurl.com"
}
```

**Responses**

- `200 OK` — returns the updated object
- `400 Bad Request` — invalid input
- `404 Not Found` — alias does not exist
- `500 Internal Server Error`

**`400` Response Body**

```json
{
  "on_item": "url",
  "errors": [
    "has to start with 'http://' or 'https://' and can not contain any whitespaces"
  ]
}
```

---

### Delete Redirect

```
DELETE /api/redirects/{alias}
```

**Path Parameters**

| Parameter | Description         |
| --------- | ------------------- |
| `alias`   | The alias to delete |

**Responses**

- `204 No Content` — redirect deleted
- `404 Not Found` — alias does not exist
- `500 Internal Server Error`

---

### Follow Redirect

```
GET /{alias}
```

Redirects the client to the URL registered under the given alias.

**Path Parameters**

| Parameter | Description         |
| --------- | ------------------- |
| `alias`   | The alias to follow |

**Responses**

- `307 Found` — redirects to the registered URL
- `404 Not Found` — alias does not exist
- `500 Internal Server Error`

---

## Health Check

### Health Check

```
GET /healthcheck
```

**Response `200 OK`**
