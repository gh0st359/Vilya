import maplibregl from "maplibre-gl";
import { events, alerts } from "./store";
import { invoke } from "@tauri-apps/api/core";

const map = new maplibregl.Map({
  container: "mapContainer",
  style: "https://demotiles.maplibre.org/style.json",
  center: [0, 20],
  zoom: 2
});

events.subscribe((evs) => {
  const fc = {
    type: "FeatureCollection",
    features: evs.map((e:any)=>({
      type:"Feature",
      properties:{ id:e.id, title:e.title, class:e.class, severity:e.severity },
      geometry:{ type:"Point", coordinates:[e.lon,e.lat] }
    }))
  };
  if (!map.getSource("events")) {
    map.addSource("events",{ type:"geojson", data: fc, cluster:true, clusterRadius:40 });
    map.addLayer({ id:"events-pts", type:"circle", source:"events",
      paint:{ "circle-radius":["interpolate",["linear"],["get","severity"],0,4,1,10] }});
  } else {
    (map.getSource("events") as any).setData(fc);
  }
});

async function refreshAlerts() {
  const b = map.getBounds();
  const bbox: [number,number,number,number] = [b.getWest(), b.getSouth(), b.getEast(), b.getNorth()];
  const now = Math.floor(Date.now()/1000);
  const list = await invoke("query_alerts", { minx: bbox[0], miny: bbox[1], maxx: bbox[2], maxy: bbox[3], nowAfter: now }) as any[];
  alerts.set(list);

  const fc = {
    type: "FeatureCollection",
    features: list.map(a=>{
      if (a.geojson && a.geojson !== "null") {
        try {
          const g = JSON.parse(a.geojson);
          return { type:"Feature", properties:{ id:a.id, severity:a.severity, headline:a.headline }, geometry:g };
        } catch {}
      }
      const [minx,miny,maxx,maxy] = a.bbox;
      return {
        type:"Feature",
        properties:{ id:a.id, severity:a.severity, headline:a.headline },
        geometry:{ type:"Polygon", coordinates:[[[minx,miny],[maxx,miny],[maxx,maxy],[minx,maxy],[minx,miny]]]}
      };
    })
  };
  if (!map.getSource("alerts")) {
    map.addSource("alerts", { type:"geojson", data: fc });
    map.addLayer({ id:"alerts-fill", type:"fill", source:"alerts",
      paint:{
        "fill-color": ["case",
          ["==", ["get","severity"], "Extreme"], "#b30000",
          ["==", ["get","severity"], "Severe"], "#e34a33",
          ["==", ["get","severity"], "Moderate"], "#fdbb84",
          ["==", ["get","severity"], "Minor"], "#fee8c8",
          "#cccccc"
        ],
        "fill-opacity": 0.25
      }
    });
    map.addLayer({ id:"alerts-outline", type:"line", source:"alerts", paint:{ "line-width": 1 }});
  } else {
    (map.getSource("alerts") as any).setData(fc);
  }
}

map.on("moveend", refreshAlerts);
map.on("load", refreshAlerts);

export default map;
