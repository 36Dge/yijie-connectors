package webexecutor

import (
	"crypto/tls"
	"crypto/x509"
	"errors"
	wire "github.com/36Dge/yijie-connectors/internal/contracts/builtinweb"
	"net/http"
	"net/url"
	"os"
	"time"
)

const APIIdentity = wire.APIIdentity
const ExecutorIdentity = wire.ExecutorIdentity

// ManagedIdentity only accepts the operator-provided private PKI. It never
// trusts forwarded HTTP headers or an end-user delegation as service identity.
func ManagedIdentity(certPath, keyPath, caPath, peerURI string, server bool) (*tls.Config, error) {
	certificate, err := tls.LoadX509KeyPair(certPath, keyPath)
	if err != nil {
		return nil, errors.New("web service certificate unavailable")
	}
	ca, err := os.ReadFile(caPath)
	if err != nil {
		return nil, errors.New("web service trust unavailable")
	}
	roots := x509.NewCertPool()
	if !roots.AppendCertsFromPEM(ca) {
		return nil, errors.New("web service trust invalid")
	}
	config := &tls.Config{MinVersion: tls.VersionTLS13, Certificates: []tls.Certificate{certificate}, RootCAs: roots}
	if server {
		config.ClientAuth = tls.RequireAndVerifyClientCert
		config.ClientCAs = roots
	}
	config.VerifyConnection = func(state tls.ConnectionState) error {
		if len(state.VerifiedChains) == 0 || len(state.PeerCertificates) == 0 {
			return errors.New("web peer unavailable")
		}
		for _, uri := range state.PeerCertificates[0].URIs {
			if uri.String() == peerURI {
				return nil
			}
		}
		return errors.New("web peer not permitted")
	}
	return config, nil
}
func ManagedClient(config *tls.Config) *http.Client {
	return &http.Client{Transport: &http.Transport{TLSClientConfig: config, DisableKeepAlives: true, ResponseHeaderTimeout: 45 * time.Second}, Timeout: 45 * time.Second, CheckRedirect: func(*http.Request, []*http.Request) error { return http.ErrUseLastResponse }}
}
func ServiceOrigin(raw string) (string, error) {
	u, err := url.Parse(raw)
	if err != nil || u.Scheme != "https" || u.Host == "" || u.User != nil || u.RawQuery != "" || u.Fragment != "" || (u.Path != "" && u.Path != "/") {
		return "", errors.New("web executor origin invalid")
	}
	u.Path = ""
	return u.String(), nil
}
