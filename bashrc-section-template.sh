source ENV_FILE

git config --global alias.pull-all 'pull --recurse-submodules && submodule foreach "git checkout main && git pull origin main"'
alias pull="current_dir=$(pwd); cd MG_HOME; git pull-all; cd $current_dir"
alias kube="kubectl"
alias mg="cd MG_HOME"
alias ac="cd ABRCITY_HOME"
alias mf="cd MG_HOME/mffabric"
alias infra="cd MG_HOME/infra"
alias sdk="cd MG_HOME/mfsdk"
alias kit="cd MG_HOME/mfkitpy"
alias chief="cd MG_HOME/mfchief"
alias control="cd MG_HOME/mfcontrol"
alias cloud="cd MG_HOME/mfcloud"
alias store="cd MG_HOME/mfstore"
alias control_ui="cd MG_HOME/mfcontrol_ui"
alias cloud_ui="cd MG_HOME/mfcloud_ui"
alias store_ui="cd MG_HOME/mfstore_ui"
alias apply="kubectl apply -f deployment.yaml"
alias reset_db="curl -X POST http://bahram:11901e637ad043c2654e4691520bc065fd@jenkins.abrcity.com/job/reset_db/build?token=da39a3ee5e6b4b0d3255bfef95601890afd80709"
alias kill_postgres="kubectl delete pods -l app=postgres -n mf-storage"
alias kill_agents="kubectl delete pods --all --namespace=mf-agents"
alias delete_workspaces="sudo rm -rf MG_WORKDIR/nfsserver/nfs_null/workspaces/*"

function run_script() {
    local current_dir=$(pwd)
    local script_dir=$(dirname "$1")
    local script_name=$(basename "$1")

    cd "$script_dir" || cd "$current_dir" || return

    ./"$script_name" "${@:2}"
    cd "$current_dir" || return
}

function abr() {
    run_script "ABRCITY_HOME/abrcity.sh" "$@"
}

function mginit() {
    run_script "MG_HOME/init.sh" "$@"
}

function jen() {
    run_script "MG_HOME/jen.sh" "$@"
}

function sync() {
    run_script "MG_HOME/sync.sh" "$@"
}

