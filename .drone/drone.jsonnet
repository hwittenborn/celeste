local deploy() = {
    name: "publish",
    kind: "pipeline",
    type: "docker",
    trigger: {branch: ["main"]},
    steps: [
        {
            name: "run-tests",
            image: "proget.makedeb.org/docker/makedeb/makedeb:ubuntu-kinetic",
            commands: [
                "sudo chown 'makedeb:makedeb' ./ -R",
                ".drone/scripts/run-tests.sh"
            ]
        },

        {
            name: "create-release",
            image: "proget.makedeb.org/docker/makedeb/makedeb:ubuntu-kinetic",
            environment: {
                github_api_key: {from_secret: "github_api_key"}
            },
            commands: [".drone/scripts/create-release.sh"]
        },

        {
            name: "publish-mpr",
            image: "proget.makedeb.org/docker/makedeb/makedeb:ubuntu-jammy",
            environment: {
                ssh_key: {from_secret: "ssh_key"}
            },
            commands: [".drone/scripts/publish-mpr.sh"]
        }
    ]
};

[deploy()]
