import groovy.json.JsonOutput

def toKebabCase(String str) {
     return str.replaceAll(/\B[A-Z]/) { '-' + it }.toLowerCase()
}

def setBuildStatus(message, context, state) {
    def nodeName = env.NODE_NAME
    container(nodeName.startsWith('techatom-jenkins-template') ? 'techatom-jenkins-builder' : 'techatom-jenkins-haskell-binary-compiler') {
        def repoName = scm.getUserRemoteConfigs()[0].getUrl().replaceAll(/(git\@github\.com:)|(\.git)/, '')
        def commit = sh(script: "git rev-parse HEAD", returnStdout: true).trim()
        withCredentials([string(credentialsId: 'tapro-labs-github-token', variable: 'TOKEN')]) {
            httpRequest (
                acceptType: 'APPLICATION_JSON',
                contentType: 'APPLICATION_JSON',
                httpMode: 'POST',
                customHeaders: [
                    [
                        name: 'Authorization',
                        value: "token $TOKEN"
                    ]
                ],
                requestBody: JsonOutput.toJson([
                    description: message,
                    state: state,
                    context: context,
                    target_url: env.BUILD_URL
                ]),
                url: "https://api.github.com/repos/$repoName/statuses/$commit"
            )
        }
    }
}

def startStage(String description, Closure code) {
    try {
      setBuildStatus("Building!", toKebabCase(description), "pending")
      stage(description) { code.call() }
      setBuildStatus("Build Successful!", toKebabCase(description), "success")
    } catch (Exception e) {
      setBuildStatus("Build Failed!", toKebabCase(description), "error")
      throw e
    }
}

def dockerRegistry = env.TECHATOM_DOCKER_REGISTRY
def dockerImagePrefix = env.DOCKER_IMAGE_PREFIX
def dockerBackendImageName = "$dockerImagePrefix/ots/ots-backend"
def dockerFrontendImageName = "$dockerImagePrefix/ots/ots-frontend"

podTemplate(name: 'techatom-jenkins-template') {
    node ('techatom-jenkins-template'){
        def gitBranch = env.BRANCH_NAME
        def environment = "staging"
        int buildNumber = env.BUILD_NUMBER as int

        // if this is our initial build we skip it
        // as we are doing a manual build to make github plugin know about the job
        if (buildNumber <= 1) {
            container('techatom-jenkins-builder') {
              stage('Checkout') {
                  checkout scm
              }
            }

            return
        }

        if (gitBranch == 'master') {
            environment = "production"
        }

        container('techatom-jenkins-builder') {
            stage('Checkout') {
                checkout scm
            }

            startStage('Install Dependencies') {
                sh "cd frontend && yarn --network-timeout 600000 --cache-folder=/tmp/.local-yarn-cache"
            }

            startStage('Build packages') {
              env.NODE_OPTIONS = '"--max-old-space-size=768"';
              sh 'cd frontend && cp .env.example .env && yarn build'
            }
        }

        container('techatom-jenkins-docker-builder-and-deployer') {
            startStage("Building docker images") {
              sh "docker build -f Dockerfile.frontend  -t ${dockerFrontendImageName}:${environment} ."
            }

            startStage("Push frontend docker image") {
                withCredentials([file(credentialsId: 'jenkins-techatom-gcloud-credential', variable: 'GC_KEY')]) {
                    sh 'gcloud auth activate-service-account --key-file=${GC_KEY}'
                    sh 'gcloud config set project tapro-labs'
                    sh 'gcloud auth configure-docker'

                    docker.withRegistry(dockerRegistry) {
                        def frontend = docker.image("${dockerFrontendImageName}:${environment}")
                        frontend.push(environment)
                        frontend.push(env.BUILD_TAG)
                    }
                }
            }
        }
    }
}


podTemplate(name: 'techatom-haskell-build-template') {
    node ('techatom-haskell-build-template'){
        def gitBranch = env.BRANCH_NAME
        def environment = "staging"
        int buildNumber = env.BUILD_NUMBER as int

        // if this is our initial build we skip it
        // as we are doing a manual build to make github plugin know about the job
        if (buildNumber <= 1) {
            container('techatom-jenkins-haskell-binary-compiler') {
              stage('Checkout') {
                  checkout scm
              }
            }

            return
        }

        if (gitBranch == 'master') {
            environment = "production"
        }

        container('techatom-jenkins-haskell-binary-compiler') {
            stage('Checkout') {
                checkout scm
            }

            startStage("Install project dependencies and build") {
              sh "cd ./backend && hpack && cabal update && cabal build -j1"
            }

            startStage("Copy binary to dist folder") {
              sh "cd ./backend/ && mkdir _build && cp \$(cabal exec which ots-server) ./_build/ots-server"
            }
        }

        container('techatom-jenkins-docker-builder-and-deployer') {
          startStage("Building docker images") {
            sh "docker build -f Dockerfile.backend -t ${dockerBackendImageName}:${environment} ."
          }

          startStage("Push backend docker image") {
              withCredentials([file(credentialsId: 'jenkins-techatom-gcloud-credential', variable: 'GC_KEY')]) {
                  sh 'gcloud auth activate-service-account --key-file=${GC_KEY}'
                  sh 'gcloud config set project tapro-labs'
                  sh 'gcloud auth configure-docker'

                  docker.withRegistry(dockerRegistry) {
                      def backend = docker.image("${dockerBackendImageName}:${environment}")
                      backend.push(environment)
                      backend.push(env.BUILD_TAG)
                  }
              }
          }

          startStage('Deploy to cluster both frontend and backend') {
              def kubernetesServerUrl = env.KUBERNETES_SERVER_URL
              def deploymentNamespace = env.DEPLOYMENT_NAMESPACE

              withKubeConfig([credentialsId: 'jenkins-kubernetes-service-account', serverUrl: kubernetesServerUrl]) {
                sh "kubectl set image deployment/ots-backend -n ${deploymentNamespace} ots-backend=${dockerBackendImageName}:${env.BUILD_TAG}"
                sh "kubectl set image deployment/ots-frontend -n ${deploymentNamespace} ots-frontend=${dockerFrontendImageName}:${env.BUILD_TAG}"
              }
          }
        }
    }
}
