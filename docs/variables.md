# BoxMux Variable System Guide

The BoxMux Variable System enables dynamic configuration, template-driven interfaces, and environment-specific deployments through hierarchical variable resolution.

## Table of Contents

- [Overview](#overview)
- [Variable Syntax](#variable-syntax)
- [Hierarchical Precedence](#hierarchical-precedence)
- [Practical Examples](#practical-examples)
- [Advanced Patterns](#advanced-patterns)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

## Overview

BoxMux variables provide these capabilities:

- Create reusable configurations that adapt to different environments
- Reduce duplication through hierarchical inheritance
- Enable template-driven deployments with dynamic content
- Integrate with existing environment variables
- Provide fallback values for robust configuration management

## Variable Syntax

### Basic Patterns

```yaml
# Standard variable substitution
title: '${VARIABLE_NAME}'

# Variable with default value
content: '${DATABASE_HOST:localhost}'

# Variable with empty default
script: ['echo "Value: ${OPTIONAL_VAR:}"']

# Legacy environment variable support
command: '$HOME/scripts/deploy.sh'
```

### Supported Fields

Variables work in all string and string array fields:

```yaml
- id: 'dynamic_panel'
  title: '${SERVICE_NAME} Monitor'          # Panel titles
  content: 'Status: ${SERVICE_STATUS}'      # Panel content
  script:                                   # Script commands
    - 'systemctl status ${SERVICE_NAME}'
    - 'journalctl -u ${SERVICE_NAME} -n 10'
  redirect_output: '${SERVICE_NAME}_logs'   # Output redirection
  choices:
    - content: 'Restart ${SERVICE_NAME}'    # Choice labels
      script: ['systemctl restart ${SERVICE_NAME}'] # Choice scripts
```

## Hierarchical Precedence

Variables are resolved in strict hierarchical order, allowing fine-grained control:

### Precedence Order (Highest to Lowest)

1. **Panel-specific variables** - Most granular control
2. **Parent panel variables** - Inherited through panel hierarchy
3. **Layout-level variables** - Layout scope (future enhancement)
4. **Application-global variables** - App-wide scope
5. **Environment variables** - System fallback
6. **Default values** - Built-in fallbacks

### Inheritance Example

```yaml
app:
  variables:
    ENVIRONMENT: "production"        # App-level: available everywhere
    DEFAULT_PORT: "8080"
    
  layouts:
    - id: 'services'
      children:
        - id: 'web_tier'
          variables:
            TIER: "frontend"         # Parent level: inherited by children
            DEFAULT_PORT: "80"       # Overrides app-level DEFAULT_PORT
          children:
            - id: 'nginx'
              variables:
                SERVICE: "nginx"     # Child level: highest precedence
                PORT: "443"          # Overrides parent DEFAULT_PORT
              title: '${SERVICE} (${TIER}) - ${ENVIRONMENT}'
              # Resolves to: "nginx (frontend) - production"
              script:
                - 'echo "Starting ${SERVICE} on port ${PORT:${DEFAULT_PORT}}"'
                # Resolves to: "Starting nginx on port 443"
                
            - id: 'apache'
              variables:
                SERVICE: "apache2"
              title: '${SERVICE} (${TIER}) - ${ENVIRONMENT}'
              # Resolves to: "apache2 (frontend) - production"
              script:
                - 'echo "Starting ${SERVICE} on port ${PORT:${DEFAULT_PORT}}"'
                # Resolves to: "Starting apache2 on port 80" (uses parent DEFAULT_PORT)
```

## Practical Examples

### Environment-Specific Configuration

Create a single configuration that works across multiple environments:

```yaml
app:
  variables:
    # Override these via environment variables for different deployments
    ENVIRONMENT: "development"
    API_BASE_URL: "http://localhost:3000"
    DATABASE_URL: "postgres://localhost:5432/myapp_dev"
    LOG_LEVEL: "debug"
    
  layouts:
    - id: 'deployment_status'
      title: 'Deployment Status - ${ENVIRONMENT}'
      children:
        - id: 'api_health'
          variables:
            SERVICE_NAME: "API Gateway"
          title: '${SERVICE_NAME} Health Check'
          script:
            - 'echo "Environment: ${ENVIRONMENT}"'
            - 'echo "Checking API at: ${API_BASE_URL}"'
            - 'curl -f ${API_BASE_URL}/health || echo "API Down"'
            
        - id: 'database_status'
          variables:
            SERVICE_NAME: "Database"
          title: '${SERVICE_NAME} Connection'
          script:
            - 'echo "Testing connection to: ${DATABASE_URL}"'
            - 'pg_isready -d "${DATABASE_URL}" && echo "Connected" || echo "Failed"'
```

**Deploy to different environments:**

```bash
# Development
./boxmux my-config.yaml

# Staging
ENVIRONMENT="staging" API_BASE_URL="https://api-staging.company.com" \
DATABASE_URL="postgres://staging-db:5432/myapp" ./boxmux my-config.yaml

# Production
ENVIRONMENT="production" API_BASE_URL="https://api.company.com" \
DATABASE_URL="postgres://prod-db:5432/myapp" LOG_LEVEL="info" ./boxmux my-config.yaml
```

### Multi-Service Monitoring Dashboard

```yaml
app:
  variables:
    MONITORING_USER: "monitor"
    SSH_KEY_PATH: "~/.ssh/monitoring_key"
    LOG_RETENTION_DAYS: "7"
    
  layouts:
    - id: 'infrastructure_overview'
      title: 'Infrastructure Monitoring'
      children:
        # Web servers section
        - id: 'web_servers'
          variables:
            SERVER_TYPE: "web"
            DEFAULT_PORT: "80"
          children:
            - id: 'web1'
              variables:
                HOSTNAME: "web1.company.com"
                SERVICE: "nginx"
                PORT: "443"
              title: '${SERVICE}@${HOSTNAME}:${PORT}'
              script:
                - 'ssh -i ${SSH_KEY_PATH} ${MONITORING_USER}@${HOSTNAME} "systemctl is-active ${SERVICE}"'
                - 'ssh -i ${SSH_KEY_PATH} ${MONITORING_USER}@${HOSTNAME} "ss -tulpn | grep :${PORT}"'
                
            - id: 'web2'
              variables:
                HOSTNAME: "web2.company.com"
                SERVICE: "apache2"
                # PORT not defined, will use parent DEFAULT_PORT (80)
              title: '${SERVICE}@${HOSTNAME}:${PORT:${DEFAULT_PORT}}'
              script:
                - 'ssh -i ${SSH_KEY_PATH} ${MONITORING_USER}@${HOSTNAME} "systemctl is-active ${SERVICE}"'
                - 'ssh -i ${SSH_KEY_PATH} ${MONITORING_USER}@${HOSTNAME} "ss -tulpn | grep :${PORT:${DEFAULT_PORT}}"'
                
        # Database servers section
        - id: 'database_servers'
          variables:
            SERVER_TYPE: "database"
            DEFAULT_PORT: "5432"
          children:
            - id: 'db_primary'
              variables:
                HOSTNAME: "db1.company.com"
                ROLE: "primary"
                SERVICE: "postgresql"
              title: '${SERVICE} ${ROLE}@${HOSTNAME}'
              script:
                - 'ssh -i ${SSH_KEY_PATH} ${MONITORING_USER}@${HOSTNAME} "sudo -u postgres psql -c \"SELECT version();\""'
                - 'ssh -i ${SSH_KEY_PATH} ${MONITORING_USER}@${HOSTNAME} "sudo -u postgres psql -c \"SELECT pg_is_in_recovery();\""'
```

### Template-Driven Deployment Pipeline

```yaml
app:
  variables:
    DEPLOYMENT_BRANCH: "main"
    DOCKER_REGISTRY: "registry.company.com"
    NAMESPACE: "default"
    REPLICAS: "2"
    
  layouts:
    - id: 'deployment_pipeline'
      title: 'Deployment Pipeline - ${DEPLOYMENT_BRANCH}'
      children:
        - id: 'build_stage'
          variables:
            STAGE: "build"
            IMAGE_TAG: "${DEPLOYMENT_BRANCH}-${BUILD_ID:latest}"
          title: '${STAGE} Stage'
          script:
            - 'echo "Building from branch: ${DEPLOYMENT_BRANCH}"'
            - 'git checkout ${DEPLOYMENT_BRANCH}'
            - 'docker build -t ${DOCKER_REGISTRY}/myapp:${IMAGE_TAG} .'
            - 'docker push ${DOCKER_REGISTRY}/myapp:${IMAGE_TAG}'
            
        - id: 'deploy_frontend'
          variables:
            COMPONENT: "frontend"
            PORT: "3000"
            HEALTH_PATH: "/"
          title: 'Deploy ${COMPONENT}'
          script:
            - 'echo "Deploying ${COMPONENT} to ${NAMESPACE}"'
            - 'kubectl set image deployment/${COMPONENT} ${COMPONENT}=${DOCKER_REGISTRY}/myapp:${IMAGE_TAG:latest} -n ${NAMESPACE}'
            - 'kubectl scale deployment/${COMPONENT} --replicas=${REPLICAS} -n ${NAMESPACE}'
            - 'kubectl rollout status deployment/${COMPONENT} -n ${NAMESPACE}'
            - 'echo "Health check: http://${COMPONENT}:${PORT}${HEALTH_PATH}"'
            
        - id: 'deploy_backend'
          variables:
            COMPONENT: "backend"
            PORT: "8080"
            HEALTH_PATH: "/api/health"
          title: 'Deploy ${COMPONENT}'
          script:
            - 'echo "Deploying ${COMPONENT} to ${NAMESPACE}"'
            - 'kubectl set image deployment/${COMPONENT} ${COMPONENT}=${DOCKER_REGISTRY}/myapp:${IMAGE_TAG:latest} -n ${NAMESPACE}'
            - 'kubectl scale deployment/${COMPONENT} --replicas=${REPLICAS} -n ${NAMESPACE}'
            - 'kubectl rollout status deployment/${COMPONENT} -n ${NAMESPACE}'
            - 'echo "Health check: http://${COMPONENT}:${PORT}${HEALTH_PATH}"'
```

## Additional Patterns

### Conditional Logic with Defaults

```yaml
# Use environment-specific settings with intelligent defaults
app:
  variables:
    # Development defaults
    DEBUG_MODE: "true"
    REPLICA_COUNT: "1"
    RESOURCE_LIMITS: "false"
    
  layouts:
    - id: 'app_deployment'
      children:
        - id: 'application'
          script:
            # Use production values if set, otherwise development defaults
            - 'echo "Debug mode: ${DEBUG_MODE}"'
            - 'echo "Replicas: ${REPLICA_COUNT}"'
            - 'echo "Resource limits: ${ENABLE_RESOURCE_LIMITS:${RESOURCE_LIMITS}}"'
            - |
              if [ "${DEBUG_MODE}" = "true" ]; then
                echo "Running in debug mode"
              else
                echo "Running in production mode"
              fi
```

### Dynamic Service Discovery

```yaml
app:
  variables:
    CONSUL_ENDPOINT: "http://consul.service.consul:8500"
    SERVICE_DISCOVERY: "consul"
    
  layouts:
    - id: 'service_mesh'
      children:
        - id: 'service_registry'
          variables:
            QUERY_PATH: "/v1/catalog/services"
          title: 'Service Registry (${SERVICE_DISCOVERY})'
          script:
            - 'curl -s ${CONSUL_ENDPOINT}${QUERY_PATH} | jq "keys"'
            
        - id: 'service_health'
          variables:
            SERVICE_NAME: "web-api"
            QUERY_PATH: "/v1/health/service"
          title: '${SERVICE_NAME} Health'
          script:
            - 'curl -s ${CONSUL_ENDPOINT}${QUERY_PATH}/${SERVICE_NAME} | jq ".[].Checks[].Status"'
```

### Multi-Environment Configuration Matrix

```yaml
app:
  variables:
    # Base configuration
    APP_NAME: "myapp"
    DEFAULT_MEMORY: "512Mi"
    DEFAULT_CPU: "0.5"
    
  layouts:
    - id: 'environment_matrix'
      children:
        - id: 'development'
          variables:
            ENV: "dev"
            REPLICAS: "1"
            MEMORY_LIMIT: "256Mi"
            CPU_LIMIT: "0.25"
          title: '${APP_NAME}-${ENV} (${REPLICAS} replicas)'
          script:
            - 'echo "Environment: ${ENV}"'
            - 'echo "Resources: CPU=${CPU_LIMIT}, Memory=${MEMORY_LIMIT}"'
            
        - id: 'staging'
          variables:
            ENV: "staging"
            REPLICAS: "2"
            # Uses DEFAULT_MEMORY and DEFAULT_CPU from app level
          title: '${APP_NAME}-${ENV} (${REPLICAS} replicas)'
          script:
            - 'echo "Environment: ${ENV}"'
            - 'echo "Resources: CPU=${CPU_LIMIT:${DEFAULT_CPU}}, Memory=${MEMORY_LIMIT:${DEFAULT_MEMORY}}"'
            
        - id: 'production'
          variables:
            ENV: "prod"
            REPLICAS: "5"
            MEMORY_LIMIT: "1Gi"
            CPU_LIMIT: "1.0"
          title: '${APP_NAME}-${ENV} (${REPLICAS} replicas)'
          script:
            - 'echo "Environment: ${ENV}"'
            - 'echo "Resources: CPU=${CPU_LIMIT}, Memory=${MEMORY_LIMIT}"'
```

## Best Practices

### 1. Use Meaningful Variable Names

```yaml
# Good: Descriptive and scoped
variables:
  DATABASE_CONNECTION_STRING: "postgres://localhost:5432/myapp"
  API_GATEWAY_ENDPOINT: "https://api.company.com"
  LOG_RETENTION_DAYS: "30"

# Avoid: Generic or ambiguous names
variables:
  URL: "https://api.company.com"      # Too generic
  CONFIG: "some_value"                # Unclear purpose
  X: "30"                             # Meaningless
```

### 2. Provide Sensible Defaults

```yaml
# Always provide fallback values for optional configuration
script:
  - 'echo "Timeout: ${REQUEST_TIMEOUT:30}s"'
  - 'echo "Retries: ${MAX_RETRIES:3}"'
  - 'echo "Log level: ${LOG_LEVEL:info}"'
```

### 3. Group Related Variables by Scope

```yaml
app:
  variables:
    # Global application settings
    APP_NAME: "myapp"
    VERSION: "1.0.0"
    ENVIRONMENT: "production"
    
  layouts:
    - id: 'database_tier'
      children:
        - id: 'postgres'
          variables:
            # Database-specific configuration
            DB_HOST: "postgres.internal"
            DB_PORT: "5432"
            DB_NAME: "myapp_prod"
            CONNECTION_POOL_SIZE: "10"
            
        - id: 'redis'
          variables:
            # Cache-specific configuration
            REDIS_HOST: "redis.internal"
            REDIS_PORT: "6379"
            REDIS_DB: "0"
            CACHE_TTL: "3600"
```

### 4. Use Environment Variables for Secrets

```yaml
# Never store secrets in YAML files
# Use environment variables with meaningful defaults for non-secrets
app:
  variables:
    DATABASE_HOST: "localhost"        # OK: Default host
    DATABASE_PORT: "5432"             # OK: Default port
    # DATABASE_PASSWORD: "secret123"  # NEVER: Use $DATABASE_PASSWORD instead
    
  layouts:
    - id: 'database_panel'
      script:
        # Reference secrets via environment variables
        - 'psql -h ${DATABASE_HOST} -p ${DATABASE_PORT} -U ${DATABASE_USER} ${DATABASE_NAME}'
        # $DATABASE_USER and $DATABASE_PASSWORD come from environment
```

### 5. Leverage Hierarchical Inheritance

```yaml
# Define common settings at higher levels, specifics at lower levels
app:
  variables:
    COMPANY_DOMAIN: "company.com"
    MONITORING_ENABLED: "true"
    
  layouts:
    - id: 'microservices'
      children:
        - id: 'service_group_a'
          variables:
            SERVICE_GROUP: "frontend"
            DEFAULT_PORT: "3000"
          children:
            - id: 'react_app'
              variables:
                SERVICE_NAME: "react-ui"
                # Inherits: COMPANY_DOMAIN, MONITORING_ENABLED, SERVICE_GROUP, DEFAULT_PORT
              title: '${SERVICE_NAME} (${SERVICE_GROUP})'
              
            - id: 'vue_app'
              variables:
                SERVICE_NAME: "vue-dashboard"
                PORT: "3001"  # Overrides DEFAULT_PORT for this service
              title: '${SERVICE_NAME} (${SERVICE_GROUP})'
```

## Troubleshooting

### Common Issues and Solutions

#### Issue: Variable Not Resolving

```yaml
# Problem: Variable shows as literal text
content: 'Server: ${SERVER_NAME}'
# Output: "Server: ${SERVER_NAME}"

# Solution 1: Check variable is defined
app:
  variables:
    SERVER_NAME: "prod-server"

# Solution 2: Provide a default
content: 'Server: ${SERVER_NAME:unknown}'
```

#### Issue: Nested Variable Syntax

```yaml
# Problem: Attempting nested variables (not supported)
content: 'User: ${USER:${DEFAULT_USER}}'
# Error: "Nested variable substitution is not supported"

# Solution: Use simpler fallback chain
app:
  variables:
    DEFAULT_USER: "admin"
content: 'User: ${USER:admin}'  # Use literal default
```

#### Issue: Environment Override Not Working

```yaml
# Problem: YAML variable always takes precedence
app:
  variables:
    LOG_LEVEL: "info"  # This overrides environment $LOG_LEVEL

# Solution: Only define in YAML if you want it to override environment
# Remove from YAML to let environment variables work:
script:
  - 'echo "Log level: ${LOG_LEVEL:info}"'  # Uses $LOG_LEVEL or "info"
```

#### Issue: Special Characters in Variable Values

```yaml
# Problem: Variable contains special shell characters
variables:
  PASSWORD: "my$ecret&password"
  
# Solution: Quote appropriately in scripts
script:
  - 'mysql -p"${PASSWORD}" mydb'  # Quote the variable substitution
```

### Debugging Variable Resolution

Add debug output to understand variable resolution:

```yaml
- id: 'debug_panel'
  variables:
    LOCAL_VAR: "panel_value"
  script:
    - 'echo "=== Variable Resolution Debug ==="'
    - 'echo "LOCAL_VAR: ${LOCAL_VAR}"'
    - 'echo "GLOBAL_VAR: ${GLOBAL_VAR:not_set}"'
    - 'echo "ENV_VAR: ${HOME:not_set}"'
    - 'echo "WITH_DEFAULT: ${UNDEFINED_VAR:default_value}"'
    - 'echo "================================"'
```

This will help you understand how variables are being resolved in your specific configuration.