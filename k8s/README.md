# Deploy the app on Kubernetes

## Install Docker, kubectl and helm

The first step is download and install Docker, kubectl and helm.

And then have kubectl properly configured to access your cluster on the cloud. e.g.:

* <https://cloud.google.com/kubernetes-engine/docs/how-to/cluster-access-for-kubectl>
* <https://docs.aws.amazon.com/eks/latest/userguide/create-kubeconfig.html>

Alternatively, you can start a cluster on your computer using minikube. In this case, run `minikube tunnel` to allow access to LoadBalancer services.

We will also need to an Ingress Controller, see <https://kubernetes.io/docs/concepts/services-networking/ingress-controllers/>. GCP GKE already comes with a Ingress Controller pre-configured and in case of minikube, you can enable the Ingress Controller by running `minikube addons enable ingress`.

## Optional: build the image

The default configuration will pull the public image `ghcr.io/randommm/rust-financial-platform` from the registry, but you can build the image yourself, if you wish.

One option is to fork the repo and let Github build the image for you, then,in the yaml files, change all references of `ghcr.io/randommm/rust-financial-platform` to `ghcr.io/your-github-username/rust-financial-platform`.

Another is to build the image locally: in the yaml files, change all references of `ghcr.io/randommm/rust-financial-platform` to the name of your image, e.g.: `my-rust-financial-platform`.

Now to configure Kubernetes to run the app, first build the Docker image using the provided Dockerfile:

`docker build . -t my-rust-financial-platform`

The image should be available on the Docker registry for Kubernetes to pull, or as an alternative, if using minikube, you can enter the minikube Docker context by running

`eval $(minikube -p minikube docker-env)`

and build then image there

`docker build . -t my-rust-financial-platform`

## Deploy the app with PostgreSQL database

Edit the username and password at postgres_helm_conf.yaml and rfp-app_secrets_conf.yaml

Then install Postgres on Kubernetes using Helm:

`helm repo add bitnami https://charts.bitnami.com/bitnami`

`helm repo update`

`helm install postgresql -f postgresql_helm_conf.yaml bitnami/postgresql`

And run:

`kubectl apply -f rfp_secrets_conf.yaml`

`kubectl apply -f rfp_app_conf.yaml`

`kubectl apply -f pgadmin_conf.yaml`

Check the IP address of the website with

`kubectl get ingress`

And change your /etc/hosts accordingly.

## PGAdmin

You can optionally setup the pgAdmin web server, by running:

`kubectl apply -f pgadmin_conf.yaml`

## Obtain free SSL certificates and serve a HTTPS server

This step is tricky to do on minikube since by default your app will not be exposed to the internet as minikube is not meant for production use. If you want to expose a self hosted Kubernetes to the internet, I'd recommend going for K3s or MicroK8s (which has a MetalLB as an easy to install addon).

After your DNS record has been successfully set up and the HTTP website is running correctly, you can obtain a free Let's Encrypt SSL certificates for your website. First install a cert-manager addon to your cluster using helm:

`helm install cert-manager jetstack/cert-manager --namespace cert-manager --create-namespace --set installCRDs=true`

Then you need to define a Let's Encrypt ClusterIssuer definition for your cluster:

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

It may take a few minutes for the validation to happen (add an extra if you changed the DNS record of your domain recently). You can get more details of the status and troubleshooting by running:

`kubectl describe clusterissuers`

`kubectl describe certificaterequests`
