package catalog

import (
	"encoding/json"
	"reflect"
	"testing"
)

func TestManagedPublicCatalogContractAndSafeProjection(t *testing.T) {
	revision, entries, err := Read()
	if err != nil || revision != 7 || len(entries) != 58 {
		t.Fatalf("catalog unavailable: revision=%d entries=%d error=%v", revision, len(entries), err)
	}
	counts := map[string]int{}
	for _, item := range entries {
		if item.ServiceId == "taobao-flash-sale-retail" || item.ServiceId == "doukou-doctor" {
			t.Fatal("retired service remains in active catalog")
		}
		counts[string(item.CategoryId)]++
	}
	want := map[string]int{"knowledge_docs": 2, "ecommerce_retail": 5, "cross_border_ecommerce": 10, "data_analytics": 2, "productivity": 14, "industry_data": 16, "marketing": 9}
	if !reflect.DeepEqual(counts, want) {
		t.Fatalf("catalog categories differ: %v", counts)
	}
	var raw struct{ Catalog []map[string]json.RawMessage }
	if err := json.Unmarshal(source, &raw); err != nil {
		t.Fatal(err)
	}
	allowed := map[string]bool{"serviceId": true, "serverName": true, "displayName": true, "categoryId": true, "categoryLabel": true, "description": true, "iconAssetId": true, "transport": true, "authMode": true, "availability": true, "blockerCodes": true}
	for _, item := range raw.Catalog {
		for key := range item {
			if !allowed[key] {
				t.Fatalf("public directory exposes non-public field %q", key)
			}
		}
	}
}
