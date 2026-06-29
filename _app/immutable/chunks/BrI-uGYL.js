import"./DsnmJJEf.js";import"./8zndd0xi.js";import{f as z,s,a as Z,b as J,d as n,r as a}from"./E9AOERj2.js";import{h as t}from"./Cc2oKZLy.js";const b={title:"Variable System Guide",description:"Complete guide to BoxMux variable system - hierarchical variables, environment integration, template-driven configuration, and dynamic deployments"},{title:os,description:ls}=b;var ss=z('<h2>Table of Contents</h2> <ul><li><a href="#overview">Overview</a></li> <li><a href="#variable-syntax">Variable Syntax</a></li> <li><a href="#hierarchical-precedence">Hierarchical Precedence</a></li> <li><a href="#practical-examples">Practical Examples</a></li> <li><a href="#advanced-patterns">Advanced Patterns</a></li> <li><a href="#best-practices">Best Practices</a></li> <li><a href="#troubleshooting">Troubleshooting</a></li></ul> <h2>Overview</h2> <p>BoxMux variables provide these capabilities:</p> <ul><li>Create reusable configurations that adapt to different environments</li> <li>Reduce duplication through hierarchical inheritance</li> <li>Enable template-driven deployments with dynamic content</li> <li>Integrate with existing environment variables</li> <li>Provide fallback values for robust configuration management</li></ul> <h2>Variable Syntax</h2> <h3>Basic Patterns</h3> <pre class="language-yaml"><!></pre> <h3>Supported Fields</h3> <p>Variables work in all string and string array fields:</p> <pre class="language-yaml"><!></pre> <h2>Hierarchical Precedence</h2> <p>Variables are resolved in strict hierarchical order, allowing fine-grained control:</p> <h3>Precedence Order (Highest to Lowest)</h3> <ol><li><strong>Box-specific variables</strong> - Most granular control</li> <li><strong>Parent box variables</strong> - Inherited through box hierarchy</li> <li><strong>Layout-level variables</strong> - Layout scope (future enhancement)</li> <li><strong>Application-global variables</strong> - App-wide scope</li> <li><strong>Environment variables</strong> - System fallback</li> <li><strong>Default values</strong> - Built-in fallbacks</li></ol> <h3>Inheritance Example</h3> <pre class="language-yaml"><!></pre> <h2>Practical Examples</h2> <h3>Environment-Specific Configuration</h3> <p>Create a single configuration that works across multiple environments:</p> <pre class="language-yaml"><!></pre> <p><strong>Deploy to different environments:</strong></p> <pre class="language-bash"><!></pre> <h3>Multi-Service Monitoring Dashboard</h3> <pre class="language-yaml"><!></pre> <h3>Template-Driven Deployment Pipeline</h3> <pre class="language-yaml"><!></pre> <h2>Advanced Patterns</h2> <h3>Conditional Logic with Defaults</h3> <pre class="language-yaml"><!></pre> <h3>Dynamic Service Discovery</h3> <pre class="language-yaml"><!></pre> <h3>Multi-Environment Configuration Matrix</h3> <pre class="language-yaml"><!></pre> <h2>Best Practices</h2> <h3>1. Use Meaningful Variable Names</h3> <pre class="language-yaml"><!></pre> <h3>2. Provide Sensible Defaults</h3> <pre class="language-yaml"><!></pre> <h3>3. Group Related Variables by Scope</h3> <pre class="language-yaml"><!></pre> <h3>4. Use Environment Variables for Secrets</h3> <pre class="language-yaml"><!></pre> <h3>5. Leverage Hierarchical Inheritance</h3> <pre class="language-yaml"><!></pre> <h2>Troubleshooting</h2> <h3>Common Issues and Solutions</h3> <h4>Issue: Variable Not Resolving</h4> <pre class="language-yaml"><!></pre> <h4>Issue: Nested Variables Not Supported</h4> <pre class="language-yaml"><!></pre> <h4>Issue: Environment Variable Override Not Working</h4> <pre class="language-bash"><!></pre> <h4>Issue: Complex Variable Expressions</h4> <pre class="language-yaml"><!></pre> <h3>Debugging Variable Resolution</h3> <h4>Enable Debug Output</h4> <pre class="language-bash"><!></pre> <h4>Test Variable Resolution</h4> <pre class="language-yaml"><!></pre> <h4>Validate Configuration</h4> <pre class="language-bash"><!></pre>',1);function ns(P){var O=ss(),e=s(Z(O),14),I=n(e);t(I,()=>`<code class="language-yaml"><span class="token comment"># Standard variable substitution</span>
<span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'$&#123;VARIABLE_NAME&#125;'</span>

<span class="token comment"># Variable with default value</span>
<span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'$&#123;DATABASE_HOST:localhost&#125;'</span>

<span class="token comment"># Variable with empty default</span>
<span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'echo "Value: $&#123;OPTIONAL_VAR:&#125;"'</span><span class="token punctuation">]</span>

<span class="token comment"># Legacy environment variable support</span>
<span class="token key atrule">command</span><span class="token punctuation">:</span> <span class="token string">'$HOME/scripts/deploy.sh'</span></code>`),a(e);var p=s(e,6),$=n(p);t($,()=>`<code class="language-yaml"><span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'dynamic_box'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'$&#123;SERVICE_NAME&#125; Monitor'</span>          <span class="token comment"># Box titles</span>
  <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Status: $&#123;SERVICE_STATUS&#125;'</span>      <span class="token comment"># Box content</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>                                   <span class="token comment"># Script commands</span>
    <span class="token punctuation">-</span> <span class="token string">'systemctl status $&#123;SERVICE_NAME&#125;'</span>
    <span class="token punctuation">-</span> <span class="token string">'journalctl -u $&#123;SERVICE_NAME&#125; -n 10'</span>
  <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">'$&#123;SERVICE_NAME&#125;_logs'</span>   <span class="token comment"># Output redirection</span>
  <span class="token key atrule">choices</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Restart $&#123;SERVICE_NAME&#125;'</span>    <span class="token comment"># Choice labels</span>
      <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'systemctl restart $&#123;SERVICE_NAME&#125;'</span><span class="token punctuation">]</span> <span class="token comment"># Choice scripts</span></code>`),a(p);var o=s(p,12),M=n(o);t(M,()=>`<code class="language-yaml"><span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">variables</span><span class="token punctuation">:</span>
    <span class="token key atrule">ENVIRONMENT</span><span class="token punctuation">:</span> <span class="token string">"production"</span>        <span class="token comment"># App-level: available everywhere</span>
    <span class="token key atrule">DEFAULT_PORT</span><span class="token punctuation">:</span> <span class="token string">"8080"</span>
    
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'services'</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'web_tier'</span>
          <span class="token key atrule">variables</span><span class="token punctuation">:</span>
            <span class="token key atrule">TIER</span><span class="token punctuation">:</span> <span class="token string">"frontend"</span>         <span class="token comment"># Parent level: inherited by children</span>
            <span class="token key atrule">DEFAULT_PORT</span><span class="token punctuation">:</span> <span class="token string">"80"</span>       <span class="token comment"># Overrides app-level DEFAULT_PORT</span>
          <span class="token key atrule">children</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'nginx'</span>
              <span class="token key atrule">variables</span><span class="token punctuation">:</span>
                <span class="token key atrule">SERVICE</span><span class="token punctuation">:</span> <span class="token string">"nginx"</span>     <span class="token comment"># Child level: highest precedence</span>
                <span class="token key atrule">PORT</span><span class="token punctuation">:</span> <span class="token string">"443"</span>          <span class="token comment"># Overrides parent DEFAULT_PORT</span>
              <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'$&#123;SERVICE&#125; ($&#123;TIER&#125;) - $&#123;ENVIRONMENT&#125;'</span>
              <span class="token comment"># Resolves to: "nginx (frontend) - production"</span>
              <span class="token key atrule">script</span><span class="token punctuation">:</span>
                <span class="token punctuation">-</span> <span class="token string">'echo "Starting $&#123;SERVICE&#125; on port $&#123;PORT:$&#123;DEFAULT_PORT&#125;&#125;"'</span>
                <span class="token comment"># Resolves to: "Starting nginx on port 443"</span>
                
            <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'apache'</span>
              <span class="token key atrule">variables</span><span class="token punctuation">:</span>
                <span class="token key atrule">SERVICE</span><span class="token punctuation">:</span> <span class="token string">"apache2"</span>
              <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'$&#123;SERVICE&#125; ($&#123;TIER&#125;) - $&#123;ENVIRONMENT&#125;'</span>
              <span class="token comment"># Resolves to: "apache2 (frontend) - production"</span>
              <span class="token key atrule">script</span><span class="token punctuation">:</span>
                <span class="token punctuation">-</span> <span class="token string">'echo "Starting $&#123;SERVICE&#125; on port $&#123;PORT:$&#123;DEFAULT_PORT&#125;&#125;"'</span>
                <span class="token comment"># Resolves to: "Starting apache2 on port 80" (uses parent DEFAULT_PORT)</span></code>`),a(o);var l=s(o,8),C=n(l);t(C,()=>`<code class="language-yaml"><span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">variables</span><span class="token punctuation">:</span>
    <span class="token comment"># Override these via environment variables for different deployments</span>
    <span class="token key atrule">ENVIRONMENT</span><span class="token punctuation">:</span> <span class="token string">"development"</span>
    <span class="token key atrule">API_BASE_URL</span><span class="token punctuation">:</span> <span class="token string">"http://localhost:3000"</span>
    <span class="token key atrule">DATABASE_URL</span><span class="token punctuation">:</span> <span class="token string">"postgres://localhost:5432/myapp_dev"</span>
    <span class="token key atrule">LOG_LEVEL</span><span class="token punctuation">:</span> <span class="token string">"debug"</span>
    
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'deployment_status'</span>
      <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Deployment Status - $&#123;ENVIRONMENT&#125;'</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'api_health'</span>
          <span class="token key atrule">variables</span><span class="token punctuation">:</span>
            <span class="token key atrule">SERVICE_NAME</span><span class="token punctuation">:</span> <span class="token string">"API Gateway"</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'$&#123;SERVICE_NAME&#125; Health Check'</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> <span class="token string">'echo "Environment: $&#123;ENVIRONMENT&#125;"'</span>
            <span class="token punctuation">-</span> <span class="token string">'echo "Checking API at: $&#123;API_BASE_URL&#125;"'</span>
            <span class="token punctuation">-</span> <span class="token string">'curl -f $&#123;API_BASE_URL&#125;/health || echo "API Down"'</span>
            
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'database_status'</span>
          <span class="token key atrule">variables</span><span class="token punctuation">:</span>
            <span class="token key atrule">SERVICE_NAME</span><span class="token punctuation">:</span> <span class="token string">"Database"</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'$&#123;SERVICE_NAME&#125; Connection'</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> <span class="token string">'echo "Testing connection to: $&#123;DATABASE_URL&#125;"'</span>
            <span class="token punctuation">-</span> <span class="token string">'pg_isready -d "$&#123;DATABASE_URL&#125;" &amp;&amp; echo "Connected" || echo "Failed"'</span></code>`),a(l);var c=s(l,4),f=n(c);t(f,()=>`<code class="language-bash"><span class="token comment"># Development</span>
./boxmux my-config.yaml

<span class="token comment"># Staging</span>
<span class="token assign-left variable">ENVIRONMENT</span><span class="token operator">=</span><span class="token string">"staging"</span> <span class="token assign-left variable">API_BASE_URL</span><span class="token operator">=</span><span class="token string">"https://api-staging.company.com"</span> <span class="token punctuation"></span>
<span class="token assign-left variable">DATABASE_URL</span><span class="token operator">=</span><span class="token string">"postgres://staging-db:5432/myapp"</span> ./boxmux my-config.yaml

<span class="token comment"># Production</span>
<span class="token assign-left variable">ENVIRONMENT</span><span class="token operator">=</span><span class="token string">"production"</span> <span class="token assign-left variable">API_BASE_URL</span><span class="token operator">=</span><span class="token string">"https://api.company.com"</span> <span class="token punctuation"></span>
<span class="token assign-left variable">DATABASE_URL</span><span class="token operator">=</span><span class="token string">"postgres://prod-db:5432/myapp"</span> <span class="token assign-left variable">LOG_LEVEL</span><span class="token operator">=</span><span class="token string">"info"</span> ./boxmux my-config.yaml</code>`),a(c);var i=s(c,4),D=n(i);t(D,()=>`<code class="language-yaml"><span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">variables</span><span class="token punctuation">:</span>
    <span class="token key atrule">MONITORING_USER</span><span class="token punctuation">:</span> <span class="token string">"monitor"</span>
    <span class="token key atrule">SSH_KEY_PATH</span><span class="token punctuation">:</span> <span class="token string">"~/.ssh/monitoring_key"</span>
    <span class="token key atrule">LOG_RETENTION_DAYS</span><span class="token punctuation">:</span> <span class="token string">"7"</span>
    
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'infrastructure_overview'</span>
      <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Infrastructure Monitoring'</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token comment"># Web servers section</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'web_servers'</span>
          <span class="token key atrule">variables</span><span class="token punctuation">:</span>
            <span class="token key atrule">SERVER_TYPE</span><span class="token punctuation">:</span> <span class="token string">"web"</span>
            <span class="token key atrule">DEFAULT_PORT</span><span class="token punctuation">:</span> <span class="token string">"80"</span>
          <span class="token key atrule">children</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'web1'</span>
              <span class="token key atrule">variables</span><span class="token punctuation">:</span>
                <span class="token key atrule">HOSTNAME</span><span class="token punctuation">:</span> <span class="token string">"web1.company.com"</span>
                <span class="token key atrule">SERVICE</span><span class="token punctuation">:</span> <span class="token string">"nginx"</span>
                <span class="token key atrule">PORT</span><span class="token punctuation">:</span> <span class="token string">"443"</span>
              <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'$&#123;SERVICE&#125;@$&#123;HOSTNAME&#125;:$&#123;PORT&#125;'</span>
              <span class="token key atrule">script</span><span class="token punctuation">:</span>
                <span class="token punctuation">-</span> <span class="token string">'ssh -i $&#123;SSH_KEY_PATH&#125; $&#123;MONITORING_USER&#125;@$&#123;HOSTNAME&#125; "systemctl is-active $&#123;SERVICE&#125;"'</span>
                <span class="token punctuation">-</span> <span class="token string">'ssh -i $&#123;SSH_KEY_PATH&#125; $&#123;MONITORING_USER&#125;@$&#123;HOSTNAME&#125; "ss -tulpn | grep :$&#123;PORT&#125;"'</span>
                
            <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'web2'</span>
              <span class="token key atrule">variables</span><span class="token punctuation">:</span>
                <span class="token key atrule">HOSTNAME</span><span class="token punctuation">:</span> <span class="token string">"web2.company.com"</span>
                <span class="token key atrule">SERVICE</span><span class="token punctuation">:</span> <span class="token string">"apache2"</span>
                <span class="token comment"># PORT not defined, will use parent DEFAULT_PORT (80)</span>
              <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'$&#123;SERVICE&#125;@$&#123;HOSTNAME&#125;:$&#123;PORT:$&#123;DEFAULT_PORT&#125;&#125;'</span>
              <span class="token key atrule">script</span><span class="token punctuation">:</span>
                <span class="token punctuation">-</span> <span class="token string">'ssh -i $&#123;SSH_KEY_PATH&#125; $&#123;MONITORING_USER&#125;@$&#123;HOSTNAME&#125; "systemctl is-active $&#123;SERVICE&#125;"'</span>
                <span class="token punctuation">-</span> <span class="token string">'ssh -i $&#123;SSH_KEY_PATH&#125; $&#123;MONITORING_USER&#125;@$&#123;HOSTNAME&#125; "ss -tulpn | grep :$&#123;PORT:$&#123;DEFAULT_PORT&#125;&#125;"'</span>
                
        <span class="token comment"># Database servers section</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'database_servers'</span>
          <span class="token key atrule">variables</span><span class="token punctuation">:</span>
            <span class="token key atrule">SERVER_TYPE</span><span class="token punctuation">:</span> <span class="token string">"database"</span>
            <span class="token key atrule">DEFAULT_PORT</span><span class="token punctuation">:</span> <span class="token string">"5432"</span>
          <span class="token key atrule">children</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'db_primary'</span>
              <span class="token key atrule">variables</span><span class="token punctuation">:</span>
                <span class="token key atrule">HOSTNAME</span><span class="token punctuation">:</span> <span class="token string">"db1.company.com"</span>
                <span class="token key atrule">ROLE</span><span class="token punctuation">:</span> <span class="token string">"primary"</span>
                <span class="token key atrule">SERVICE</span><span class="token punctuation">:</span> <span class="token string">"postgresql"</span>
              <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'$&#123;SERVICE&#125; $&#123;ROLE&#125;@$&#123;HOSTNAME&#125;'</span>
              <span class="token key atrule">script</span><span class="token punctuation">:</span>
                <span class="token punctuation">-</span> <span class="token string">'ssh -i $&#123;SSH_KEY_PATH&#125; $&#123;MONITORING_USER&#125;@$&#123;HOSTNAME&#125; "sudo -u postgres psql -c "SELECT version();""'</span>
                <span class="token punctuation">-</span> <span class="token string">'ssh -i $&#123;SSH_KEY_PATH&#125; $&#123;MONITORING_USER&#125;@$&#123;HOSTNAME&#125; "sudo -u postgres psql -c "SELECT pg_is_in_recovery();""'</span></code>`),a(i);var k=s(i,4),L=n(k);t(L,()=>`<code class="language-yaml"><span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">variables</span><span class="token punctuation">:</span>
    <span class="token key atrule">DEPLOYMENT_BRANCH</span><span class="token punctuation">:</span> <span class="token string">"main"</span>
    <span class="token key atrule">DOCKER_REGISTRY</span><span class="token punctuation">:</span> <span class="token string">"registry.company.com"</span>
    <span class="token key atrule">NAMESPACE</span><span class="token punctuation">:</span> <span class="token string">"default"</span>
    <span class="token key atrule">REPLICAS</span><span class="token punctuation">:</span> <span class="token string">"2"</span>
    
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'deployment_pipeline'</span>
      <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Deployment Pipeline - $&#123;DEPLOYMENT_BRANCH&#125;'</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'build_stage'</span>
          <span class="token key atrule">variables</span><span class="token punctuation">:</span>
            <span class="token key atrule">STAGE</span><span class="token punctuation">:</span> <span class="token string">"build"</span>
            <span class="token key atrule">IMAGE_TAG</span><span class="token punctuation">:</span> <span class="token string">"$&#123;DEPLOYMENT_BRANCH&#125;-$&#123;BUILD_ID:latest&#125;"</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'$&#123;STAGE&#125; Stage'</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> <span class="token string">'echo "Building from branch: $&#123;DEPLOYMENT_BRANCH&#125;"'</span>
            <span class="token punctuation">-</span> <span class="token string">'git checkout $&#123;DEPLOYMENT_BRANCH&#125;'</span>
            <span class="token punctuation">-</span> <span class="token string">'docker build -t $&#123;DOCKER_REGISTRY&#125;/myapp:$&#123;IMAGE_TAG&#125; .'</span>
            <span class="token punctuation">-</span> <span class="token string">'docker push $&#123;DOCKER_REGISTRY&#125;/myapp:$&#123;IMAGE_TAG&#125;'</span>
            
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'deploy_frontend'</span>
          <span class="token key atrule">variables</span><span class="token punctuation">:</span>
            <span class="token key atrule">COMPONENT</span><span class="token punctuation">:</span> <span class="token string">"frontend"</span>
            <span class="token key atrule">PORT</span><span class="token punctuation">:</span> <span class="token string">"3000"</span>
            <span class="token key atrule">HEALTH_PATH</span><span class="token punctuation">:</span> <span class="token string">"/"</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Deploy $&#123;COMPONENT&#125;'</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> <span class="token string">'echo "Deploying $&#123;COMPONENT&#125; to $&#123;NAMESPACE&#125;"'</span>
            <span class="token punctuation">-</span> <span class="token string">'kubectl set image deployment/$&#123;COMPONENT&#125; $&#123;COMPONENT&#125;=$&#123;DOCKER_REGISTRY&#125;/myapp:$&#123;IMAGE_TAG:latest&#125; -n $&#123;NAMESPACE&#125;'</span>
            <span class="token punctuation">-</span> <span class="token string">'kubectl scale deployment/$&#123;COMPONENT&#125; --replicas=$&#123;REPLICAS&#125; -n $&#123;NAMESPACE&#125;'</span>
            <span class="token punctuation">-</span> <span class="token string">'kubectl rollout status deployment/$&#123;COMPONENT&#125; -n $&#123;NAMESPACE&#125;'</span>
            <span class="token punctuation">-</span> <span class="token string">'echo "Health check: http://$&#123;COMPONENT&#125;:$&#123;PORT&#125;$&#123;HEALTH_PATH&#125;"'</span>
            
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'deploy_backend'</span>
          <span class="token key atrule">variables</span><span class="token punctuation">:</span>
            <span class="token key atrule">COMPONENT</span><span class="token punctuation">:</span> <span class="token string">"backend"</span>
            <span class="token key atrule">PORT</span><span class="token punctuation">:</span> <span class="token string">"8080"</span>
            <span class="token key atrule">HEALTH_PATH</span><span class="token punctuation">:</span> <span class="token string">"/api/health"</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Deploy $&#123;COMPONENT&#125;'</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> <span class="token string">'echo "Deploying $&#123;COMPONENT&#125; to $&#123;NAMESPACE&#125;"'</span>
            <span class="token punctuation">-</span> <span class="token string">'kubectl set image deployment/$&#123;COMPONENT&#125; $&#123;COMPONENT&#125;=$&#123;DOCKER_REGISTRY&#125;/myapp:$&#123;IMAGE_TAG:latest&#125; -n $&#123;NAMESPACE&#125;'</span>
            <span class="token punctuation">-</span> <span class="token string">'kubectl scale deployment/$&#123;COMPONENT&#125; --replicas=$&#123;REPLICAS&#125; -n $&#123;NAMESPACE&#125;'</span>
            <span class="token punctuation">-</span> <span class="token string">'kubectl rollout status deployment/$&#123;COMPONENT&#125; -n $&#123;NAMESPACE&#125;'</span>
            <span class="token punctuation">-</span> <span class="token string">'echo "Health check: http://$&#123;COMPONENT&#125;:$&#123;PORT&#125;$&#123;HEALTH_PATH&#125;"'</span></code>`),a(k);var u=s(k,6),V=n(u);t(V,()=>`<code class="language-yaml"><span class="token comment"># Use environment-specific settings with intelligent defaults</span>
<span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">variables</span><span class="token punctuation">:</span>
    <span class="token comment"># Development defaults</span>
    <span class="token key atrule">DEBUG_MODE</span><span class="token punctuation">:</span> <span class="token string">"true"</span>
    <span class="token key atrule">REPLICA_COUNT</span><span class="token punctuation">:</span> <span class="token string">"1"</span>
    <span class="token key atrule">RESOURCE_LIMITS</span><span class="token punctuation">:</span> <span class="token string">"false"</span>
    
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'app_deployment'</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'application'</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token comment"># Use production values if set, otherwise development defaults</span>
            <span class="token punctuation">-</span> <span class="token string">'echo "Debug mode: $&#123;DEBUG_MODE&#125;"'</span>
            <span class="token punctuation">-</span> <span class="token string">'echo "Replicas: $&#123;REPLICA_COUNT&#125;"'</span>
            <span class="token punctuation">-</span> <span class="token string">'echo "Resource limits: $&#123;ENABLE_RESOURCE_LIMITS:$&#123;RESOURCE_LIMITS&#125;&#125;"'</span>
            <span class="token punctuation">-</span> <span class="token punctuation">|</span><span class="token scalar string">
              if [ "$&#123;DEBUG_MODE&#125;" = "true" ]; then
                echo "Running in debug mode"
              else
                echo "Running in production mode"
              fi</span></code>`),a(u);var r=s(u,4),U=n(r);t(U,()=>`<code class="language-yaml"><span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">variables</span><span class="token punctuation">:</span>
    <span class="token key atrule">CONSUL_ENDPOINT</span><span class="token punctuation">:</span> <span class="token string">"http://consul.service.consul:8500"</span>
    <span class="token key atrule">SERVICE_DISCOVERY</span><span class="token punctuation">:</span> <span class="token string">"consul"</span>
    
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'service_mesh'</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'service_registry'</span>
          <span class="token key atrule">variables</span><span class="token punctuation">:</span>
            <span class="token key atrule">QUERY_PATH</span><span class="token punctuation">:</span> <span class="token string">"/v1/catalog/services"</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Service Registry ($&#123;SERVICE_DISCOVERY&#125;)'</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> <span class="token string">'curl -s $&#123;CONSUL_ENDPOINT&#125;$&#123;QUERY_PATH&#125; | jq "keys"'</span>
            
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'service_health'</span>
          <span class="token key atrule">variables</span><span class="token punctuation">:</span>
            <span class="token key atrule">SERVICE_NAME</span><span class="token punctuation">:</span> <span class="token string">"web-api"</span>
            <span class="token key atrule">QUERY_PATH</span><span class="token punctuation">:</span> <span class="token string">"/v1/health/service"</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'$&#123;SERVICE_NAME&#125; Health'</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> <span class="token string">'curl -s $&#123;CONSUL_ENDPOINT&#125;$&#123;QUERY_PATH&#125;/$&#123;SERVICE_NAME&#125; | jq ".[].Checks[].Status"'</span></code>`),a(r);var g=s(r,4),H=n(g);t(H,()=>`<code class="language-yaml"><span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">variables</span><span class="token punctuation">:</span>
    <span class="token comment"># Base configuration</span>
    <span class="token key atrule">APP_NAME</span><span class="token punctuation">:</span> <span class="token string">"myapp"</span>
    <span class="token key atrule">DEFAULT_MEMORY</span><span class="token punctuation">:</span> <span class="token string">"512Mi"</span>
    <span class="token key atrule">DEFAULT_CPU</span><span class="token punctuation">:</span> <span class="token string">"0.5"</span>
    
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'environment_matrix'</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'development'</span>
          <span class="token key atrule">variables</span><span class="token punctuation">:</span>
            <span class="token key atrule">ENV</span><span class="token punctuation">:</span> <span class="token string">"dev"</span>
            <span class="token key atrule">REPLICAS</span><span class="token punctuation">:</span> <span class="token string">"1"</span>
            <span class="token key atrule">MEMORY_LIMIT</span><span class="token punctuation">:</span> <span class="token string">"256Mi"</span>
            <span class="token key atrule">CPU_LIMIT</span><span class="token punctuation">:</span> <span class="token string">"0.25"</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'$&#123;APP_NAME&#125;-$&#123;ENV&#125; ($&#123;REPLICAS&#125; replicas)'</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> <span class="token string">'echo "Environment: $&#123;ENV&#125;"'</span>
            <span class="token punctuation">-</span> <span class="token string">'echo "Resources: CPU=$&#123;CPU_LIMIT&#125;, Memory=$&#123;MEMORY_LIMIT&#125;"'</span>
            
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'staging'</span>
          <span class="token key atrule">variables</span><span class="token punctuation">:</span>
            <span class="token key atrule">ENV</span><span class="token punctuation">:</span> <span class="token string">"staging"</span>
            <span class="token key atrule">REPLICAS</span><span class="token punctuation">:</span> <span class="token string">"2"</span>
            <span class="token comment"># Uses DEFAULT_MEMORY and DEFAULT_CPU from app level</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'$&#123;APP_NAME&#125;-$&#123;ENV&#125; ($&#123;REPLICAS&#125; replicas)'</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> <span class="token string">'echo "Environment: $&#123;ENV&#125;"'</span>
            <span class="token punctuation">-</span> <span class="token string">'echo "Resources: CPU=$&#123;CPU_LIMIT:$&#123;DEFAULT_CPU&#125;&#125;, Memory=$&#123;MEMORY_LIMIT:$&#123;DEFAULT_MEMORY&#125;&#125;"'</span>
            
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'production'</span>
          <span class="token key atrule">variables</span><span class="token punctuation">:</span>
            <span class="token key atrule">ENV</span><span class="token punctuation">:</span> <span class="token string">"prod"</span>
            <span class="token key atrule">REPLICAS</span><span class="token punctuation">:</span> <span class="token string">"5"</span>
            <span class="token key atrule">MEMORY_LIMIT</span><span class="token punctuation">:</span> <span class="token string">"1Gi"</span>
            <span class="token key atrule">CPU_LIMIT</span><span class="token punctuation">:</span> <span class="token string">"1.0"</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'$&#123;APP_NAME&#125;-$&#123;ENV&#125; ($&#123;REPLICAS&#125; replicas)'</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> <span class="token string">'echo "Environment: $&#123;ENV&#125;"'</span>
            <span class="token punctuation">-</span> <span class="token string">'echo "Resources: CPU=$&#123;CPU_LIMIT&#125;, Memory=$&#123;MEMORY_LIMIT&#125;"'</span></code>`),a(g);var E=s(g,6),B=n(E);t(B,()=>`<code class="language-yaml"><span class="token comment"># Good: Descriptive and scoped</span>
<span class="token key atrule">variables</span><span class="token punctuation">:</span>
  <span class="token key atrule">DATABASE_CONNECTION_STRING</span><span class="token punctuation">:</span> <span class="token string">"postgres://localhost:5432/myapp"</span>
  <span class="token key atrule">API_GATEWAY_ENDPOINT</span><span class="token punctuation">:</span> <span class="token string">"https://api.company.com"</span>
  <span class="token key atrule">LOG_RETENTION_DAYS</span><span class="token punctuation">:</span> <span class="token string">"30"</span>

<span class="token comment"># Avoid: Generic or ambiguous names</span>
<span class="token key atrule">variables</span><span class="token punctuation">:</span>
  <span class="token key atrule">URL</span><span class="token punctuation">:</span> <span class="token string">"https://api.company.com"</span>      <span class="token comment"># Too generic</span>
  <span class="token key atrule">CONFIG</span><span class="token punctuation">:</span> <span class="token string">"some_value"</span>                <span class="token comment"># Unclear purpose</span>
  <span class="token key atrule">X</span><span class="token punctuation">:</span> <span class="token string">"30"</span>                             <span class="token comment"># Meaningless</span></code>`),a(E);var y=s(E,4),G=n(y);t(G,()=>`<code class="language-yaml"><span class="token comment"># Always provide fallback values for optional configuration</span>
<span class="token key atrule">script</span><span class="token punctuation">:</span>
  <span class="token punctuation">-</span> <span class="token string">'echo "Timeout: $&#123;REQUEST_TIMEOUT:30&#125;s"'</span>
  <span class="token punctuation">-</span> <span class="token string">'echo "Retries: $&#123;MAX_RETRIES:3&#125;"'</span>
  <span class="token punctuation">-</span> <span class="token string">'echo "Log level: $&#123;LOG_LEVEL:info&#125;"'</span></code>`),a(y);var m=s(y,4),x=n(m);t(x,()=>`<code class="language-yaml"><span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">variables</span><span class="token punctuation">:</span>
    <span class="token comment"># Global application settings</span>
    <span class="token key atrule">APP_NAME</span><span class="token punctuation">:</span> <span class="token string">"myapp"</span>
    <span class="token key atrule">VERSION</span><span class="token punctuation">:</span> <span class="token string">"1.0.0"</span>
    <span class="token key atrule">ENVIRONMENT</span><span class="token punctuation">:</span> <span class="token string">"production"</span>
    
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'database_tier'</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'postgres'</span>
          <span class="token key atrule">variables</span><span class="token punctuation">:</span>
            <span class="token comment"># Database-specific configuration</span>
            <span class="token key atrule">DB_HOST</span><span class="token punctuation">:</span> <span class="token string">"postgres.internal"</span>
            <span class="token key atrule">DB_PORT</span><span class="token punctuation">:</span> <span class="token string">"5432"</span>
            <span class="token key atrule">DB_NAME</span><span class="token punctuation">:</span> <span class="token string">"myapp_prod"</span>
            <span class="token key atrule">CONNECTION_POOL_SIZE</span><span class="token punctuation">:</span> <span class="token string">"10"</span>
            
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'redis'</span>
          <span class="token key atrule">variables</span><span class="token punctuation">:</span>
            <span class="token comment"># Cache-specific configuration</span>
            <span class="token key atrule">REDIS_HOST</span><span class="token punctuation">:</span> <span class="token string">"redis.internal"</span>
            <span class="token key atrule">REDIS_PORT</span><span class="token punctuation">:</span> <span class="token string">"6379"</span>
            <span class="token key atrule">REDIS_DB</span><span class="token punctuation">:</span> <span class="token string">"0"</span>
            <span class="token key atrule">CACHE_TTL</span><span class="token punctuation">:</span> <span class="token string">"3600"</span></code>`),a(m);var d=s(m,4),Y=n(d);t(Y,()=>`<code class="language-yaml"><span class="token comment"># Never store secrets in YAML files</span>
<span class="token comment"># Use environment variables with meaningful defaults for non-secrets</span>
<span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">variables</span><span class="token punctuation">:</span>
    <span class="token key atrule">DATABASE_HOST</span><span class="token punctuation">:</span> <span class="token string">"localhost"</span>        <span class="token comment"># OK: Default host</span>
    <span class="token key atrule">DATABASE_PORT</span><span class="token punctuation">:</span> <span class="token string">"5432"</span>             <span class="token comment"># OK: Default port</span>
    <span class="token comment"># DATABASE_PASSWORD: "secret123"  # NEVER: Use $DATABASE_PASSWORD instead</span>
    
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'database_box'</span>
      <span class="token key atrule">script</span><span class="token punctuation">:</span>
        <span class="token comment"># Reference secrets via environment variables</span>
        <span class="token punctuation">-</span> <span class="token string">'psql -h $&#123;DATABASE_HOST&#125; -p $&#123;DATABASE_PORT&#125; -U $&#123;DATABASE_USER&#125; $&#123;DATABASE_NAME&#125;'</span>
        <span class="token comment"># $DATABASE_USER and $DATABASE_PASSWORD come from environment</span></code>`),a(d);var _=s(d,4),w=n(_);t(w,()=>`<code class="language-yaml"><span class="token comment"># Define common settings at higher levels, specifics at lower levels</span>
<span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">variables</span><span class="token punctuation">:</span>
    <span class="token key atrule">COMPANY_DOMAIN</span><span class="token punctuation">:</span> <span class="token string">"company.com"</span>
    <span class="token key atrule">MONITORING_ENABLED</span><span class="token punctuation">:</span> <span class="token string">"true"</span>
    
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'microservices'</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'service_group_a'</span>
          <span class="token key atrule">variables</span><span class="token punctuation">:</span>
            <span class="token key atrule">SERVICE_GROUP</span><span class="token punctuation">:</span> <span class="token string">"frontend"</span>
            <span class="token key atrule">DEFAULT_PORT</span><span class="token punctuation">:</span> <span class="token string">"3000"</span>
          <span class="token key atrule">children</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'react_app'</span>
              <span class="token key atrule">variables</span><span class="token punctuation">:</span>
                <span class="token key atrule">SERVICE_NAME</span><span class="token punctuation">:</span> <span class="token string">"react-ui"</span>
                <span class="token comment"># Inherits: COMPANY_DOMAIN, MONITORING_ENABLED, SERVICE_GROUP, DEFAULT_PORT</span>
              <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'$&#123;SERVICE_NAME&#125; ($&#123;SERVICE_GROUP&#125;)'</span>
              
            <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'vue_app'</span>
              <span class="token key atrule">variables</span><span class="token punctuation">:</span>
                <span class="token key atrule">SERVICE_NAME</span><span class="token punctuation">:</span> <span class="token string">"vue-dashboard"</span>
                <span class="token key atrule">PORT</span><span class="token punctuation">:</span> <span class="token string">"3001"</span>  <span class="token comment"># Overrides DEFAULT_PORT for this service</span>
              <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'$&#123;SERVICE_NAME&#125; ($&#123;SERVICE_GROUP&#125;)'</span></code>`),a(_);var A=s(_,8),F=n(A);t(F,()=>`<code class="language-yaml"><span class="token comment"># Problem: Variable shows as literal text</span>
<span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Server: $&#123;SERVER_NAME&#125;'</span>
<span class="token comment"># Output: "Server: $&#123;SERVER_NAME&#125;"</span>

<span class="token comment"># Solution 1: Check variable is defined</span>
<span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">variables</span><span class="token punctuation">:</span>
    <span class="token key atrule">SERVER_NAME</span><span class="token punctuation">:</span> <span class="token string">"prod-server"</span>

<span class="token comment"># Solution 2: Use default value</span>
<span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Server: $&#123;SERVER_NAME:default-server&#125;'</span></code>`),a(A);var h=s(A,4),K=n(h);t(K,()=>`<code class="language-yaml"><span class="token comment"># Problem: This doesn't work</span>
<span class="token key atrule">variables</span><span class="token punctuation">:</span>
  <span class="token key atrule">BASE_URL</span><span class="token punctuation">:</span> <span class="token string">"https://api.company.com"</span>
  <span class="token key atrule">ENDPOINT</span><span class="token punctuation">:</span> <span class="token string">"$&#123;BASE_URL&#125;/v1"</span>  <span class="token comment"># Nested variable reference</span>

<span class="token comment"># Solution: Use environment variables for composition</span>
<span class="token comment"># Set in shell: export ENDPOINT="$&#123;BASE_URL&#125;/v1"</span></code>`),a(h);var v=s(h,4),j=n(v);t(j,()=>`<code class="language-bash"><span class="token comment"># Problem: Environment variable not taking precedence</span>
<span class="token builtin class-name">export</span> <span class="token assign-left variable">DATABASE_HOST</span><span class="token operator">=</span><span class="token string">"override-host"</span>
./boxmux config.yaml

<span class="token comment"># Solution 1: Check variable name matches exactly</span>
<span class="token comment"># YAML: DATABASE_HOST vs Environment: DATABASE_HOST ✓</span>
<span class="token comment"># YAML: db_host vs Environment: DATABASE_HOST ✗</span>

<span class="token comment"># Solution 2: Verify export is in same shell session</span>
<span class="token function">env</span> <span class="token operator">|</span> <span class="token function">grep</span> DATABASE_HOST  <span class="token comment"># Should show your value</span></code>`),a(v);var T=s(v,4),q=n(T);t(q,()=>`<code class="language-yaml"><span class="token comment"># Problem: Trying to do complex operations</span>
<span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'$&#123;PORT + 1000&#125;'</span>  <span class="token comment"># Not supported</span>

<span class="token comment"># Solution: Use script logic instead</span>
<span class="token key atrule">script</span><span class="token punctuation">:</span>
  <span class="token punctuation">-</span> <span class="token string">'PORT_PLUS_1000=$(($&#123;PORT:8080&#125; + 1000))'</span>
  <span class="token punctuation">-</span> <span class="token string">'echo "Adjusted port: $PORT_PLUS_1000"'</span></code>`),a(T);var R=s(T,6),W=n(R);t(W,()=>`<code class="language-bash"><span class="token comment"># Run with debug logging to see variable resolution</span>
<span class="token assign-left variable">RUST_LOG</span><span class="token operator">=</span>debug ./boxmux config.yaml <span class="token operator"><span class="token file-descriptor important">2</span>></span><span class="token file-descriptor important">&amp;1</span> <span class="token operator">|</span> <span class="token function">grep</span> <span class="token parameter variable">-i</span> variable</code>`),a(R);var S=s(R,4),Q=n(S);t(Q,()=>`<code class="language-yaml"><span class="token comment"># Create a debug box to test variable values</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'debug_vars'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Variable Debug'</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token string">'echo "All variables:"'</span>
    <span class="token punctuation">-</span> <span class="token string">'echo "APP_NAME: $&#123;APP_NAME:not_set&#125;"'</span>
    <span class="token punctuation">-</span> <span class="token string">'echo "ENVIRONMENT: $&#123;ENVIRONMENT:not_set&#125;"'</span>
    <span class="token punctuation">-</span> <span class="token string">'echo "DATABASE_HOST: $&#123;DATABASE_HOST:not_set&#125;"'</span>
    <span class="token punctuation">-</span> <span class="token string">'echo "Environment variables:"'</span>
    <span class="token punctuation">-</span> <span class="token string">'env | grep -E "(APP_NAME|ENVIRONMENT|DATABASE_HOST)"'</span></code>`),a(S);var N=s(S,4),X=n(N);t(X,()=>`<code class="language-bash"><span class="token comment"># Test configuration with known environment</span>
<span class="token assign-left variable">APP_NAME</span><span class="token operator">=</span><span class="token string">"test"</span> <span class="token assign-left variable">ENVIRONMENT</span><span class="token operator">=</span><span class="token string">"debug"</span> ./boxmux config.yaml</code>`),a(N),J(P,O)}const cs=Object.freeze(Object.defineProperty({__proto__:null,default:ns,metadata:b},Symbol.toStringTag,{value:"Module"}));export{cs as _};
