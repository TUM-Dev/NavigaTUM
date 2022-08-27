#!/usr/bin/bash

# args: "main"
# args: non-main-branch PR-number

set -e # fail on first
tmp_dir=$(mktemp -d -t navigatum-XXXXXXXXXX)
# remove the tmp dir on exit
#trap "trap - SIGTERM && rm -fr $tmp_dir" SIGINT SIGTERM EXIT
echo "Temporary files will be written to: $tmp_dir"


(
cd ./deployment || exit
if [ $1 == "main" ]; then
    echo "Deploying to production"
    rm k3s/templates/application.yaml

    cp -R k3s/* "$tmp_dir/main"
else
    echo "Deploying to PR-based environment"

    export BRANCH=$1
    export PR_NUMBER=$2
    echo "creating app for $BRANCH on pr-$PR_NUMBER"

    # In PR-based environments, someone knowing the Meili master key is not a concern
    MEILI_MASTER_KEY="$(cat /dev/urandom | tr -dc '[:alpha:]' | head -c 64|base64)"
    export MEILI_MASTER_KEY

    mkdir "$tmp_dir/pr-$PR_NUMBER"
    cat k3s/templates/application.yaml | kyml tmpl --env BRANCH --env PR_NUMBER --env MEILI_MASTER_KEY > tmp.yaml
    cp -R k3s/* "$tmp_dir/pr-$PR_NUMBER"
    mv tmp.yaml "$tmp_dir/pr-$PR_NUMBER/templates/application.yaml"
fi
)

echo "checking out deployment branch"
git checkout deployment
mv "$tmp_dir" .
git add .

# --porcelain returns nothing if there are no changes
if [ "$(git status --porcelain)" ]; then
    echo "no changes to deployment, returnning"
    exit 0
fi

# git branch deployment
if [ $1 == "main" ]; then
    git commit -m "Updated the production deployment"
else
    git commit -m "Updated the staging deployment for pr-$2"
fi

git push