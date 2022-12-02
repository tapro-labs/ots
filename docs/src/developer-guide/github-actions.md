# GitHub Actions

### Required Environment Variables

- ZONE - The zone where your project is located in [Google Cloud][gcloud_url]
- PROJECT_ID - The project id from your [Google Cloud Account][gcloud_url] 
- CLUSTER_ID - The cluster id from the k8s cluster in your [Google Cloud Account][gcloud_url] 
- K8S_SERVER_CA - K8s cluster certificate-authority-data [Reference][k8s-cluster-info-reference]
- K8S_SERVER_URL - K8s cluster server URL [Reference][k8s-cluster-info-reference]
- K8S_DEPLOYER_ACCOUNT_TOKEN - The service account token that is responsible for deployment. See [Example](#deployer-example) for creating a service account for the whole cluster.
- ENABLED_FEATURES - The features to be enabled on the backend. (Example: `slack`)
- GCLOUD_CREDENTIAL_FILE - The service account credential from you [Google Cloud][gcloud_url]

[gcloud_url]: https://en.wiktionary.org/wiki/a_picture_paints_a_thousand_words#:~:text=Reportedly%20first%20used%20by%20Frederick,large%20amount%20of%20descriptive%20text.
[k8s-cluster-info-reference]: https://kubernetes.io/docs/tasks/access-application-cluster/configure-access-multiple-clusters/#define-clusters-users-and-contexts

### Deployer Example

The example can be tweaked to work only for a specific namespace and not the whole cluster

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: github-cluster-deployer-account
  namespace: github
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: github-cluster-deployer-role
rules:
  - apiGroups: ["apps"]
    resources:
      - deployments
      - statefulsets
    verbs:
      - get
      - update
      - patch
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: github-cluster-deployer-role-binding
subjects:
  - kind: ServiceAccount
    name: github-cluster-deployer-account
    namespace: github
roleRef:
  kind: ClusterRole
  name: github-cluster-deployer-role
  apiGroup: rbac.authorization.k8s.io
```
