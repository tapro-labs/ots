package api

import (
	"encoding/json"
	"io"
	"net/http"
	"strconv"
	"strings"
	"testing"
)

func TestCreateSecret_WireFormatMatchesBackend(t *testing.T) {
	// Decode into a generic map so a typo in the struct tags is caught.
	// The backend (backend/src/app/secrets.rs) expects "secret" and
	// "expiry_seconds", and answers with "secretId".
	srv := startServer(t, func(w http.ResponseWriter, r *http.Request) {
		body, _ := io.ReadAll(r.Body)
		var got map[string]any
		if err := json.Unmarshal(body, &got); err != nil {
			t.Fatalf("request is not JSON: %v", err)
		}
		if len(got) != 2 {
			t.Errorf("expected exactly 2 fields, got %v", got)
		}
		if got["secret"] != "payload$_$" {
			t.Errorf("secret field: %v", got["secret"])
		}
		if got["expiry_seconds"] != float64(432000) {
			t.Errorf("expiry_seconds field: %v", got["expiry_seconds"])
		}
		w.Write([]byte(`{"secretId":"wire-id"}`))
	})

	id, err := CreateSecret(srv.URL+"/api", "payload$_$", 432000)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if id != "wire-id" {
		t.Errorf("got %q", id)
	}
}

func TestCreateSecret_BaseURLWithPath(t *testing.T) {
	srv := startServer(t, func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/api/secrets" {
			t.Errorf("expected /api/secrets, got %s", r.URL.Path)
		}
		w.Write([]byte(`{"secretId":"x"}`))
	})
	if _, err := CreateSecret(srv.URL+"/api//", "s", 300); err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
}

func TestCreateSecret_Non200Statuses(t *testing.T) {
	for _, status := range []int{
		http.StatusCreated, // backend answers 200; anything else is unexpected
		http.StatusBadRequest,
		http.StatusRequestEntityTooLarge,
		http.StatusTooManyRequests,
		http.StatusBadGateway,
	} {
		t.Run(http.StatusText(status), func(t *testing.T) {
			srv := startServer(t, func(w http.ResponseWriter, r *http.Request) {
				w.WriteHeader(status)
				w.Write([]byte(`{"secretId":"should-not-be-used"}`))
			})
			_, err := CreateSecret(srv.URL, "s", 300)
			if err == nil {
				t.Fatal("expected error")
			}
			if !strings.Contains(err.Error(), strconv.Itoa(status)) {
				t.Errorf("error should mention the status: %v", err)
			}
		})
	}
}

func TestCreateSecret_422_MessageInError(t *testing.T) {
	srv := startServer(t, func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusUnprocessableEntity)
		w.Write([]byte(`{"message":"Invalid expiry_seconds value: 1"}`))
	})
	_, err := CreateSecret(srv.URL, "s", 1)
	if err == nil || !strings.Contains(err.Error(), "Invalid expiry_seconds value: 1") {
		t.Fatalf("expected server message in error, got %v", err)
	}
}

func TestCreateSecret_422_NonJSONBody(t *testing.T) {
	srv := startServer(t, func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusUnprocessableEntity)
		w.Write([]byte(`<html>Unprocessable</html>`))
	})
	_, err := CreateSecret(srv.URL, "s", 1)
	if err == nil || !strings.Contains(err.Error(), "422") {
		t.Fatalf("expected generic 422 error, got %v", err)
	}
}

func TestCreateSecret_TruncatedResponseBody(t *testing.T) {
	// Announce more bytes than are sent, then drop the connection.
	srv := startServer(t, func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Length", "100")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`{"secretId":`))
		hj, ok := w.(http.Hijacker)
		if !ok {
			t.Fatal("hijacking not supported")
		}
		conn, _, _ := hj.Hijack()
		conn.Close()
	})
	_, err := CreateSecret(srv.URL, "s", 300)
	if err == nil || !strings.Contains(err.Error(), "reading response") {
		t.Fatalf("expected reading response error, got %v", err)
	}
}

func TestCreateSecret_InvalidURL(t *testing.T) {
	if _, err := CreateSecret("://not a url", "s", 300); err == nil {
		t.Fatal("expected error for an invalid base URL")
	}
	if _, err := CreateSecret("ots.example.com/api", "s", 300); err == nil {
		t.Fatal("expected error for a base URL without a scheme")
	}
}
