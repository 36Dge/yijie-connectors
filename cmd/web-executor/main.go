// web-executor is a service deployment entry, never an App sidecar.
package main

import (
	"context"
	"log/slog"
	"net"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/36Dge/yijie-connectors/internal/webexecutor"
	"github.com/google/uuid"
)

func run() error {
	address := os.Getenv("YIJIE_WEB_EXECUTOR_ADDRESS")
	if _, _, err := net.SplitHostPort(address); err != nil {
		return err
	}
	origin, err := webexecutor.ServiceOrigin(os.Getenv("YIJIE_WEB_API_ORIGIN"))
	if err != nil {
		return err
	}
	cert, key, ca := os.Getenv("YIJIE_WEB_SERVICE_CERT_FILE"), os.Getenv("YIJIE_WEB_SERVICE_KEY_FILE"), os.Getenv("YIJIE_WEB_SERVICE_CA_FILE")
	serverTLS, err := webexecutor.ManagedIdentity(cert, key, ca, webexecutor.APIIdentity, true)
	if err != nil {
		return err
	}
	clientTLS, err := webexecutor.ManagedIdentity(cert, key, ca, webexecutor.APIIdentity, false)
	if err != nil {
		return err
	}
	worker, err := webexecutor.NewWorker(os.Getenv("YIJIE_WEB_WORKER_MANIFEST"), os.Getenv("YIJIE_WEB_KIMI_KEY_FILE"))
	if err != nil {
		return err
	}
	handler := &webexecutor.Handler{Client: webexecutor.ManagedClient(clientTLS), APIOrigin: origin, Worker: worker, Instance: uuid.New()}
	server := &http.Server{Addr: address, TLSConfig: serverTLS, Handler: handler.Handler(), ReadHeaderTimeout: 5 * time.Second, ReadTimeout: 10 * time.Second, WriteTimeout: 55 * time.Second, IdleTimeout: 30 * time.Second}
	done := make(chan error, 1)
	go func() { done <- server.ListenAndServeTLS("", "") }()
	stop, cancel := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer cancel()
	select {
	case <-stop.Done():
	case err = <-done:
	}
	ctx, release := context.WithTimeout(context.Background(), 55*time.Second)
	defer release()
	shutdownErr := server.Shutdown(ctx)
	if worker.Wait(ctx) != nil {
		slog.Error("managed web executor STOP_PENDING; awaiting normal child exit")
		_ = worker.Wait(context.Background())
	}
	if shutdownErr != nil {
		return shutdownErr
	}
	if err == http.ErrServerClosed {
		return nil
	}
	return err
}
func main() {
	if err := run(); err != nil {
		slog.Error("managed web executor stopped before readiness or normal shutdown completed")
		os.Exit(1)
	}
}
