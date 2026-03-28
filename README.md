# ALICE-Navigation-SaaS

Path planning as a Service — A*, RRT*, navigation mesh construction, and dynamic obstacle management via REST API.

## Architecture

```
Client
  |
  v
API Gateway (:8221)
  |
  v
Core Engine (:8121)
  |
  +-- Path Planner (A* / D*)
  +-- RRT* Sampler
  +-- NavMesh Builder
  +-- Obstacle Registry
```

## Features

- Grid-based A* and D* Lite path planning
- Sampling-based RRT* for high-dimensional configuration spaces
- Navigation mesh generation from SDF geometry
- Dynamic obstacle insertion and removal with replanning
- Cost map layers (terrain, safety, no-fly zones)
- Path smoothing (Bezier, B-spline post-processing)

## API Endpoints

### Core Engine (port 8121)

| Method | Path | Description |
|--------|------|-------------|
| POST | /api/v1/nav/plan | Plan a path between start and goal |
| POST | /api/v1/nav/rrt | Run RRT* in a given configuration space |
| POST | /api/v1/nav/mesh | Build or update a navigation mesh |
| POST | /api/v1/nav/obstacles | Register dynamic obstacles |
| GET  | /api/v1/nav/stats | Return runtime statistics |
| GET  | /health | Health check |

### Example: Plan Path

```bash
curl -X POST http://localhost:8121/api/v1/nav/plan \
  -H 'Content-Type: application/json' \
  -d '{"start":[0.0,0.0],"goal":[10.0,10.0],"algorithm":"astar","grid_resolution":0.1}'
```

### Example: RRT*

```bash
curl -X POST http://localhost:8121/api/v1/nav/rrt \
  -H 'Content-Type: application/json' \
  -d '{"start":[0.0,0.0,0.0],"goal":[5.0,5.0,2.0],"max_iterations":5000,"step_size":0.3}'
```

## License

AGPL-3.0-or-later
