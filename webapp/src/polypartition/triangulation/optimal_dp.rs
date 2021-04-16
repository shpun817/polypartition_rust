use crate::polypartition::{Polygon, PolygonInterface};
use crate::polypartition::util::{Diagonal, distance, DPState, is_in_cone, intersects};

use std::collections::VecDeque;

pub fn triangulate_opt_vec(polys: Vec<Polygon>) -> Result<Vec<Polygon>, String> {
    let mut triangles = vec![];
    for poly in polys.iter() {
        if poly.props().is_hole {
            return Err("Input polygon cannot be a hole in optimal dp!".into());
        }
        let result = triangulate_opt(poly);
        match result {
            Ok(pieces) => triangles.extend(pieces),
            Err(e) => return Err(e.into())
        }
    }
    Ok(triangles)
}

/// Takes an arbitrary polygon.
///
/// Returns a vec of triangles.
pub fn triangulate_opt(poly: &Polygon) -> Result<Vec<Polygon>, &str> {
    if !poly.is_valid() {
        return Err("Input polygon is invalid.");
    }

    let mut triangles = vec![];
    let num_vertices = poly.num_points();

    // Just in case (should be guarded above)
    if num_vertices < 3 {
        return Err("Input polygon has less than 3 vertices");
    }
    // Trivial case
    if num_vertices == 3 {
        return Ok(vec![poly.clone()]);
    }

    let mut dp_states: Vec<Vec<DPState>> = Vec::with_capacity(num_vertices);
    dp_states.push(vec![]); // The skipped index 0
    for i in 1..num_vertices {
        dp_states.push(vec![DPState::default(); i]);
    }

    // Initialize states and visibility
    for i in 0..(num_vertices-1) {
        let p1 = poly.get_point(i);
        for j in (i+1)..num_vertices {
            dp_states[j][i].visible = true;

            if j != i+1 {
                let p2 = poly.get_point(j);

                // Visibility check
                let p3 = poly.get_point(if i == 0 {num_vertices - 1} else {i-1});
                let p4 = poly.get_point((i+1) % num_vertices);
                if !is_in_cone(&p3, &p1, &p4, &p2) {
                    dp_states[j][i].visible = false;
                    continue;
                }

                let p3 = poly.get_point(if j == 0 {num_vertices - 1} else {j-1});
                let p4 = poly.get_point((j+1) % num_vertices);
                if !is_in_cone(&p3, &p2, &p4, &p1) {
                    dp_states[j][i].visible = false;
                    continue;
                }

                for k in 0..num_vertices {
                    let p3 = poly.get_point(k);
                    let p4 = poly.get_point((k+1) % num_vertices);
                    if intersects(&p1, &p2, &p3, &p4) {
                        dp_states[j][i].visible = false;
                        break;
                    }
                }
            }
        }
    }
    dp_states[num_vertices - 1][0].visible = true;

    for gap in 2..num_vertices {
        for i in 0..(num_vertices - gap) {
            let j = i + gap;
            if !dp_states[j][i].visible {
                continue;
            }

            let mut best_vertex = None;
            let mut min_weight = f64::MAX;
            for k in (i+1)..j {
                if !dp_states[k][i].visible {
                    continue;
                }
                if !dp_states[j][k].visible {
                    continue;
                }

                let d1;
                let d2;

                if k <= i+1 {
                    d1 = 0.0;
                } else {
                    d1 = distance(&poly.get_point(i), &poly.get_point(k));
                }
                if j <= k+1 {
                    d2 = 0.0;
                } else {
                    d2 = distance(&poly.get_point(k), &poly.get_point(j));
                }

                let weight = dp_states[k][i].weight + dp_states[j][k].weight + d1 + d2;

                if best_vertex.is_none() || weight < min_weight {
                    best_vertex = Some(k);
                    min_weight = weight;
                }
            }
            if best_vertex.is_none() {
                return Err("No best vertex found");
            }

            dp_states[j][i].best_vertex = best_vertex;
            dp_states[j][i].weight = min_weight;
        }
    }

    let mut diagonals = VecDeque::<Diagonal>::new();
    diagonals.push_back(Diagonal::new(0, num_vertices - 1));
    while !diagonals.is_empty() {
        let diagonal = diagonals.pop_front().unwrap();
        let best_vertex = dp_states[diagonal.index_2][diagonal.index_1].best_vertex;
        if best_vertex.is_none() {
            return Err("No best vertex found in diagonal stage");
        }
        let best_vertex = best_vertex.unwrap();

        triangles.push(Polygon::triangle(
            poly.get_point(diagonal.index_1),
            poly.get_point(best_vertex),
            poly.get_point(diagonal.index_2),
        ));

        if best_vertex > diagonal.index_1 + 1 {
            diagonals.push_back(Diagonal::new(diagonal.index_1, best_vertex));
        }
        if diagonal.index_2 > best_vertex + 1 {
            diagonals.push_back(Diagonal::new(best_vertex, diagonal.index_2));
        }
    }

    Ok(triangles)
}