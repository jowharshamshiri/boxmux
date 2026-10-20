---
title: Variable System Guide
description: Complete guide to BoxMux variable system - hierarchical variables, environment integration, template-driven configuration, and dynamic deployments
---


## Table of Contents

- [Overview](#overview)
- [Variable Syntax](#variable-syntax)
- [Hierarchical Precedence](#hierarchical-precedence)
- [Variables in Scripts](#variables-in-scripts)
- [Practical Examples](#practical-examples)
- [Advanced Patterns](#advanced-patterns)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

## Overview

Variables let one configuration file adapt to different machines and environments without copying it:

- Define a value once and use it in titles, content, scripts, positions, colors and ids.
- Give a group of boxes shared values, and override them in single boxes.
- Read values from the environment, with a fallback when they are not set.

BoxMux replaces variable references when it loads the file, before it validates it. After loading, the configuration contains only the substituted text.

## Variable Syntax

### Basic Patterns

```yaml
- id: patterns
  position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
  variables:
    APP_NAME: Inventory
    DEFAULT_PORT: 8080
  # The value of APP_NAME
  title: '${APP_NAME}'
  content: |
    Host: ${DATABASE_HOST:localhost}
    Optional: ${OPTIONAL_VAR:}
    Port: ${PORT:${DEFAULT_PORT}}
    Write $${NAME} to use a variable
```

Unless the environment sets `DATABASE_HOST`, `OPTIONAL_VAR` or `PORT`, the box shows:

```text
Host: localhost
Optional:
Port: 8080
Write ${NAME} to use a variable
```

| Form | Result |
|------|--------|
| `${NAME}` | The value of `NAME`. If `NAME` is not defined anywhere, the text stays `${NAME}`. |
| `${NAME:default}` | The value of `NAME`, else `default`. The default may contain references. |
| `$${` | A literal `${`. |
| `$NAME` (no braces) | Not a BoxMux reference; left as written. |

A name starts with a letter or underscore, followed by letters, digits or underscores. Shell syntax that only looks like a reference is left as written: `${X:-y}`, `${X:=y}`, `${X:?y}`, `${X:+y}`, `${#list[@]}`, `${file%.txt}`, `${list[0]}`.

### Defining Variables

`variables` can be set on the app, on a layout and on a box. Values may be strings, numbers or booleans; numbers and booleans are used as text.

```yaml
app:
  variables:
    APP_NAME: "Production Dashboard"
    RETRIES: 3
  layouts:
    - id: main
      variables:
        REGION: eu-west
      children:
        - id: web
          variables:
            SERVICE: nginx
          title: '${SERVICE} in ${REGION}'
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
          script:
            - 'echo "${APP_NAME}: checking ${SERVICE} (${RETRIES} retries)"'
```

A variable's value is used exactly as written. References inside a value are not expanded, so `URL: "${HOST}/api"` gives the literal text `${HOST}/api`. Compose values where they are used instead: `'${HOST}/api'`.

### Supported Fields

Substitution applies to every string value in the file, at any depth. This includes titles, content, scripts, choice labels and scripts, ids, `redirect_output`, positions, colors, `chart_data`, `table_data`, `plugin_config`, `libs` and plugin paths.

It does not apply to:

- Mapping keys, such as the variable names themselves and the key names in `on_keypress` and `hot_keys`.
- The values inside `variables` mappings.
- Numbers and booleans. A setting that needs a number, such as `refresh_interval`, cannot take a reference: `refresh_interval: "${RATE}"` is a string and fails validation.
- Boxes, content and scripts sent later through the control socket.

```yaml
- id: '${SERVICE}_status'
  variables:
    SERVICE: database
    WIDTH: "40"
    ACCENT: cyan
  title: '${SERVICE} Monitor'
  position: {x1: "0", y1: "0", x2: "${WIDTH}", y2: 50%}
  border_color: '${ACCENT}'
  choices:
    - id: restart
      content: 'Restart ${SERVICE}'
      script:
        - systemctl restart ${SERVICE}
```

The substituted values must still be valid. If `ACCENT` were `purple`, loading would fail because `purple` is not a color.

## Hierarchical Precedence

### Precedence Order (Highest to Lowest)

1. **The box's own variables**
2. **Enclosing boxes' variables**, nearest first
3. **Layout variables**
4. **App variables**
5. **Environment variables** of the boxmux process
6. **The default** in `${NAME:default}`, when no level defines the name

A value defined in the YAML file always wins over the environment. To let the environment decide, do not define the name in `variables`; write `${NAME:fallback}` where it is used.

### Inheritance Example

```yaml
app:
  variables:
    ENVIRONMENT: "production"        # Available everywhere
    DEFAULT_PORT: "8080"

  layouts:
    - id: services
      children:
        - id: web_tier
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
          variables:
            TIER: "frontend"         # Inherited by the boxes inside
            DEFAULT_PORT: "80"       # Overrides the app value inside this box
          children:
            - id: nginx
              position: {x1: 2%, y1: 5%, x2: 49%, y2: 95%}
              variables:
                SERVICE: "nginx"
                PORT: "443"
              title: '${SERVICE} (${TIER}) - ${ENVIRONMENT}'
              # Becomes: "nginx (frontend) - production"
              script:
                - 'echo "Starting ${SERVICE} on port ${PORT:${DEFAULT_PORT}}"'
                # Becomes: echo "Starting nginx on port 443"

            - id: apache
              position: {x1: 51%, y1: 5%, x2: 98%, y2: 95%}
              variables:
                SERVICE: "apache2"
              title: '${SERVICE} (${TIER}) - ${ENVIRONMENT}'
              # Becomes: "apache2 (frontend) - production"
              script:
                - 'echo "Starting ${SERVICE} on port ${PORT:${DEFAULT_PORT}}"'
                # PORT is not defined (unless it is set in the environment),
                # so this becomes: echo "Starting apache2 on port 80"
```

## Variables in Scripts

Scripts are substituted like every other string, when the file is loaded. This matters when a script uses shell variables:

- `${NAME}` is replaced by BoxMux if `NAME` is defined in the file **or in the environment**. Otherwise it is left for the shell.
- `$NAME` without braces is never touched by BoxMux; the shell expands it when the script runs.
- `$${NAME}` always reaches the shell as `${NAME}`.

```yaml
- id: loop
  position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
  variables:
    COUNT: 3
  script: |
    # ${COUNT} is replaced at load time: the shell sees "seq 3"
    for i in $(seq ${COUNT}); do
      # ${i} is not defined in the file or the environment, so the shell expands it
      echo "Run ${i}"
    done
    # USER is set in the environment: ${USER} is replaced at load time,
    # $USER and $${USER} are expanded by the shell when the script runs
    echo "${USER} $USER $${USER}"
```

Use `$NAME` or `$${NAME}` for loop variables and other shell variables whose names might also exist in the environment, such as `USER`, `HOME` or `PATH`.

## Practical Examples

### Environment-Specific Configuration

One file for several environments. The names are not defined in `variables`, so the environment decides and the defaults apply for development:

```yaml
app:
  layouts:
    - id: deployment_status
      title: 'Deployment Status - ${ENVIRONMENT:development}'
      children:
        - id: api_health
          title: 'API Health Check'
          position: {x1: 0%, y1: 0%, x2: 50%, y2: 100%}
          refresh_interval: 10000
          script:
            - 'echo "Environment: ${ENVIRONMENT:development}"'
            - 'echo "Checking API at: ${API_BASE_URL:http://localhost:3000}"'
            - 'curl -fsS ${API_BASE_URL:http://localhost:3000}/health || echo "API down"'

        - id: database_status
          title: 'Database Connection'
          position: {x1: 50%, y1: 0%, x2: 100%, y2: 100%}
          script:
            - 'pg_isready -d "${DATABASE_URL:postgres://localhost:5432/myapp_dev}" && echo "Connected"'
```

**Run it for different environments:**

```bash
# Development: the defaults apply
boxmux status.yaml

# Staging
export ENVIRONMENT=staging API_BASE_URL=https://api-staging.example.com
export DATABASE_URL=postgres://staging-db:5432/myapp
boxmux status.yaml
```

### Multi-Service Monitoring Dashboard

```yaml
app:
  variables:
    MONITORING_USER: "monitor"
    SSH_KEY_PATH: "~/.ssh/monitoring_key"

  layouts:
    - id: infrastructure_overview
      title: 'Infrastructure Monitoring'
      children:
        - id: web_servers
          title: 'Web Servers'
          position: {x1: 0%, y1: 0%, x2: 50%, y2: 100%}
          variables:
            DEFAULT_PORT: "80"
          children:
            - id: web1
              variables:
                HOSTNAME: "web1.example.com"
                SERVICE: "nginx"
                PORT: "443"
              title: '${SERVICE}@${HOSTNAME}:${PORT}'
              position: {x1: 3%, y1: 5%, x2: 97%, y2: 48%}
              refresh_interval: 30000
              script:
                - 'ssh -i ${SSH_KEY_PATH} ${MONITORING_USER}@${HOSTNAME} "systemctl is-active ${SERVICE}"'

            - id: web2
              variables:
                HOSTNAME: "web2.example.com"
                SERVICE: "apache2"
              # PORT is not defined here, so the default from web_servers applies
              title: '${SERVICE}@${HOSTNAME}:${PORT:${DEFAULT_PORT}}'
              position: {x1: 3%, y1: 52%, x2: 97%, y2: 95%}
              refresh_interval: 30000
              script:
                - 'ssh -i ${SSH_KEY_PATH} ${MONITORING_USER}@${HOSTNAME} "systemctl is-active ${SERVICE}"'

        - id: db_primary
          variables:
            HOSTNAME: "db1.example.com"
            SERVICE: "postgresql"
          title: '${SERVICE}@${HOSTNAME}'
          position: {x1: 50%, y1: 0%, x2: 100%, y2: 100%}
          script:
            - 'ssh -i ${SSH_KEY_PATH} ${MONITORING_USER}@${HOSTNAME} "systemctl is-active ${SERVICE}"'
```

### Template-Driven Deployment Pipeline

```yaml
app:
  variables:
    DOCKER_REGISTRY: "registry.example.com"
    NAMESPACE: "default"
    REPLICAS: 2

  layouts:
    - id: deployment_pipeline
      title: 'Deployment Pipeline - ${DEPLOYMENT_BRANCH:main}'
      children:
        - id: stages
          title: Stages
          position: {x1: 0%, y1: 0%, x2: 30%, y2: 100%}
          tab_order: 1
          choices:
            - id: build
              content: 'Build ${DEPLOYMENT_BRANCH:main}'
              execution_mode: Thread
              redirect_output: log
              script:
                - 'docker build -t ${DOCKER_REGISTRY}/myapp:${DEPLOYMENT_BRANCH:main}-${BUILD_ID:latest} .'
                - 'docker push ${DOCKER_REGISTRY}/myapp:${DEPLOYMENT_BRANCH:main}-${BUILD_ID:latest}'
            - id: deploy_frontend
              content: 'Deploy frontend'
              execution_mode: Thread
              redirect_output: log
              script:
                - 'kubectl scale deployment/frontend --replicas=${REPLICAS} -n ${NAMESPACE}'
                - 'kubectl rollout status deployment/frontend -n ${NAMESPACE}'

        - id: log
          title: Log
          position: {x1: 30%, y1: 0%, x2: 100%, y2: 100%}
          auto_scroll_bottom: true
          content: 'Pick a stage'
```

## Advanced Patterns

### Defaults for Optional Settings

```yaml
- id: application
  position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
  script:
    - 'echo "Debug mode: ${DEBUG_MODE:false}"'
    - 'echo "Replicas: ${REPLICA_COUNT:1}"'
    - |
      if [ "${DEBUG_MODE:false}" = "true" ]; then
        echo "Running in debug mode"
      else
        echo "Running in normal mode"
      fi
```

### Chained Defaults

A default can refer to another variable, which can have its own default:

```yaml
- id: limits
  position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
  variables:
    DEFAULT_MEMORY: "512Mi"
  content: 'Memory: ${MEMORY_LIMIT:${DEFAULT_MEMORY}}, CPU: ${CPU_LIMIT:${DEFAULT_CPU:0.5}}'
  # Without MEMORY_LIMIT, CPU_LIMIT and DEFAULT_CPU in the environment:
  # "Memory: 512Mi, CPU: 0.5"
```

### Shared Values per Group of Boxes

Variables on an enclosing box apply to all the boxes inside it:

```yaml
app:
  variables:
    APP_NAME: "myapp"
  layouts:
    - id: environment_matrix
      children:
        - id: staging
          title: '${APP_NAME}-${ENV}'
          position: {x1: 0%, y1: 0%, x2: 50%, y2: 100%}
          variables:
            ENV: "staging"
            REPLICAS: "2"
          children:
            - id: staging_pods
              position: {x1: 3%, y1: 5%, x2: 97%, y2: 95%}
              script:
                - 'kubectl get pods -n ${APP_NAME}-${ENV}'

        - id: production
          title: '${APP_NAME}-${ENV}'
          position: {x1: 50%, y1: 0%, x2: 100%, y2: 100%}
          variables:
            ENV: "prod"
            REPLICAS: "5"
          children:
            - id: production_pods
              position: {x1: 3%, y1: 5%, x2: 97%, y2: 95%}
              script:
                - 'kubectl get pods -n ${APP_NAME}-${ENV}'
```

## Best Practices

### 1. Use Meaningful Variable Names

```yaml
# Good: descriptive and specific
variables:
  DATABASE_CONNECTION_STRING: "postgres://localhost:5432/myapp"
  API_GATEWAY_ENDPOINT: "https://api.example.com"
  LOG_RETENTION_DAYS: 30
```

Avoid generic names such as `URL` or `X`. Also avoid names of common environment variables (`USER`, `HOME`, `PATH`) unless you mean to override them in the file.

### 2. Provide Defaults

```yaml
script:
  - 'echo "Timeout: ${REQUEST_TIMEOUT:30}s"'
  - 'echo "Retries: ${MAX_RETRIES:3}"'
  - 'echo "Log level: ${LOG_LEVEL:info}"'
```

### 3. Define Variables at the Narrowest Scope

Put a value on the box or group of boxes that uses it. Put only values that every layout needs on the app.

### 4. Keep Secrets Out of the File

BoxMux substitutes every reference when it loads the file, so a secret written as `${DB_PASSWORD}` ends up in the script text. Let the shell read secrets when the script runs instead, with `$DB_PASSWORD` (no braces):

```yaml
- id: database_box
  position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
  variables:
    DATABASE_HOST: "localhost"
  script:
    - 'PGPASSWORD="$DB_PASSWORD" psql -h ${DATABASE_HOST} -U "$DB_USER" -c "select 1"'
```

## Troubleshooting

### Variable Not Resolving

```yaml
content: 'Server: ${SERVER_NAME}'
# Shows: "Server: ${SERVER_NAME}"
```

`SERVER_NAME` is not defined on the box, an enclosing box, the layout, the app or in the environment. Define it, or give a default: `${SERVER_NAME:default-server}`. Check that the name is spelled the same way, including case.

### A Variable's Value Shows `${...}`

```yaml
variables:
  BASE_URL: "https://api.example.com"
  ENDPOINT: "${BASE_URL}/v1"     # Not expanded: ENDPOINT is literally "${BASE_URL}/v1"
```

Values are not expanded. Write `'${BASE_URL}/v1'` where you need it.

### The Environment Does Not Override a Value

```bash
DATABASE_HOST=override-host boxmux config.yaml
```

If the file defines `DATABASE_HOST` in any `variables`, that value wins. Remove it from `variables` and use `${DATABASE_HOST:localhost}` where it is needed.

### A Shell Variable Was Replaced

```yaml
script:
  - 'for USER in alice bob; do echo ${USER}; done'
# Prints your login name twice: ${USER} was replaced at load time
```

Write `$USER` or `$${USER}` so the shell expands it.

### Validation Fails After Substitution

Errors are reported for the substituted values. For example, `border_color: '${ACCENT}'` with `ACCENT: purple` fails with `"purple" is not one of [...]`. A variable with a value that is not a string, number or boolean is also an error, for example `app.variables.LIST must be a string, number or boolean`.

### Test Variable Values

A box can show the values it sees:

```yaml
- id: debug_vars
  title: 'Variable Debug'
  position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
  content: |
    APP_NAME: ${APP_NAME:not set}
    ENVIRONMENT: ${ENVIRONMENT:not set}
    DATABASE_HOST: ${DATABASE_HOST:not set}
```
