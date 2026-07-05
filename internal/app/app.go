package app

import (
	"encoding/json"
	"net/http"
	"os"
)

const ServiceName = "yijie-connectors"

type Config struct {
	Environment string `json:"environment"`
	Port        string `json:"port"`
}

type Status struct {
	Service     string   `json:"service"`
	Status      string   `json:"status"`
	Environment string   `json:"environment"`
	Platforms   []string `json:"platforms"`
}

func LoadConfig() Config {
	return Config{
		Environment: env("YIJIE_ENV", "local"),
		Port:        env("YIJIE_CONNECTORS_PORT", "18081"),
	}
}

func NewHandler(config Config) http.Handler {
	status := Status{
		Service:     ServiceName,
		Status:      "ok",
		Environment: config.Environment,
		Platforms:   []string{"amazon", "temu", "shopee", "tiktok-shop"},
	}

	mux := http.NewServeMux()
	mux.HandleFunc("/healthz", jsonHandler(status))
	mux.HandleFunc("/readyz", jsonHandler(map[string]string{"status": "ready"}))
	mux.HandleFunc("/v1/status", jsonHandler(status))
	return mux
}

func jsonHandler(payload any) http.HandlerFunc {
	return func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		if err := json.NewEncoder(w).Encode(payload); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
		}
	}
}

func env(key, fallback string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return fallback
}
