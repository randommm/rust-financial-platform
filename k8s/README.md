Deploy the app on Kubernetes
============================
The first step is download and install Docker, kubectl and helm.

And then have kubectl properly configured to access your cluster on the cloud. e.g.:
* https://cloud.google.com/kubernetes-engine/docs/how-to/cluster-access-for-kubectl
* https://docs.aws.amazon.com/eks/latest/userguide/create-kubeconfig.html

Alternatively, you can start a cluster on your computer using minikube. In this case, run `minikube tunnel` to allow access to LoadBalancer services.

Now to configure Kubernetes to run the app on Nginx, first build the Docker image using the provided Dockerfile:

`docker build . -t rust-financial-platform:v1`

if you a name other than "rust-financial-platform:v1" in the command above, then update the line "image: rust-financial-platform:v1" in rfp-app_conf.yaml with the chosen name.

The image should be available on the Docker registry for kubernetes to pull, or as an alternative, you can enter the minikube Docker context by running

`eval $(minikube -p minikube docker-env)`

and build then image there

`docker build . -t rust-financial-platform:v1`

You can now go the section for database that you wish to use to deploy your app.

We will also need to an Ingress Controller, see https://kubernetes.io/docs/concepts/services-networking/ingress-controllers/. GCP GKE already comes with a Ingress Controller pre-configured and in case of minikube, you can enable the Ingress Controller by running `minikube addons enable ingress`.

Deploy using PostgreSQL
========================

Edit the username and password at postgres_helm_conf.yaml and rfp-app_secrets_conf.yaml

Then install Postgres on Kubernetes using Helm:

`helm repo add bitnami https://charts.bitnami.com/bitnami`

`helm repo update`

`helm install postgresql -f postgresql_helm_conf.yaml bitnami/postgresql`

And run:

`kubectl apply -f rfp-app_secrets_conf.yaml`

`kubectl apply -f rfp-app_conf.yaml`

`kubectl apply -f pgadmin_conf.yaml`

Check the IP address of the website with

`kubectl get ingress`

You can optinally setup the pgAdmin webserver, by running:

`kubectl apply -f pgadmin_conf.yaml`

Obtain free SSL certificates and serve a HTTPS server
=======================================================

After your DNS record has been sucessfully setuped and the HTTP website is running correctly, you can install obtain free Let's Encrypt SSL certificates for your website, first install a cert-manager addon to your cluster using helm:

`helm install cert-manager jetstack/cert-manager --namespace cert-manager --create-namespace --set installCRDs=true`

Then you can

    apiVersion: cert-manager.io/v1
    kind: ClusterIssuer
    metadata:
      name: letsencrypt-prod
    spec:
      acme:
        server: https://acme-v02.api.letsencrypt.org/directory
        email: youremail@yourdomain.com
        privateKeySecretRef:
          name: letsencrypt-prod
        solvers:
        - http01:
            ingress:
              name: rfp-app-ingress


An then add the cert-manager.io annotation plus the tls spec to your ingress policy:

    apiVersion: networking.k8s.io/v1
    kind: Ingress
    metadata:
      name: rfp-app-ingress
      annotations:
        cert-manager.io/cluster-issuer: letsencrypt-prod
    spec:
      tls:
      - hosts:
        - yourdomain.com
        secretName: covid-app-tls
      rules:
        - host: yourdomain.com
          http:
            paths:
              - path: /
                pathType: Prefix
                backend:
                  service:
                    name: rfp-app-service
                    port:
                      number: 80

From then on, cert-manager should be able to automatically obtain and renew your SSL certificates. You can check the status of your certificate by running:

`kubectl get certificate`

It may take a few minutes for the validation to happen (add an extra if you changed the DNS record of your domain recently). You can get more details of the status and throubleshoot by running:

`kubectl describe clusterissuers`

`kubectl describe certificaterequests`
