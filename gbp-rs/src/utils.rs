
use crate::Timestep;

// struct LookaheadParams {
//     horizon: usize,
//     multiple: usize,
// }

// TODO: take a struct as argument for better names

/// Compute the timesteps at which variables in the planned path are placed.
/// For a lookahead_multiple of 3, variables are spaced at timesteps:
/// 0,  1, 2, 3,  5, 7, 9, 12, 15, 18, ...
/// e.g. variables ar in groups of size lookahead_multiple.
/// The spacing within a group increases by one each time (1, for the first group, 2 for the second etc.)
/// Seems convoluted, but the reasoning was:
/// - The first variable should always be at 1 timestep from the current state (0).
/// - The first few variables should be close together in time
/// - The variables should all be at integer timesteps, but the spacing should sort of increase exponentially.
/// ## Example:
/// ```rust
/// let lookahead_horizon = 20;
/// let lookahead_multiple = 3;
/// assert_eq!(
///     get_variable_timesteps(lookahead_horizon, lookahead_multiple),
///     vec![0, 1, 2, 3, 5, 7, 9, 12, 15, 18, 20]
/// );
/// ```
pub fn get_variable_timesteps(lookahead_horizon: u32, lookahead_multiple: u32) -> Vec<Timestep> {
    // A visual argument is given for the estimate of the initial capacity in this desmos graph.
    // https://www.desmos.com/calculator/lxxsuqtgdq
    let estimated_capacity = (2.5 * f32::sqrt(lookahead_horizon as f32)) as usize;
    let mut timesteps = Vec::<Timestep>::with_capacity(estimated_capacity);
    timesteps.push(0);
    // TODO: use std::iter::successors instead
    for i in 1..lookahead_horizon {
        // timesteps[i] = timesteps[i-1] + (i - 1) / lookahead_multiple + 1;
        let ts = timesteps[timesteps.len() - 1] + ((i - 1) / lookahead_multiple) + 1;
        if ts >= lookahead_horizon {
            timesteps.push(lookahead_horizon);
            break;
        }
        timesteps.push(ts);
    }

    timesteps
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_get_variable_timesteps() {
        let lookahead_horizon = 30;
        let lookahead_multiple = 3;

        assert_eq!(
            get_variable_timesteps(lookahead_horizon, lookahead_multiple),
            vec![0, 1, 2, 3, 5, 7, 9, 12, 15, 18, 22, 26, 30]
        );

        let lookahead_horizon = 60;
        let lookahead_multiple = 3;
        assert_eq!(
            get_variable_timesteps(lookahead_horizon, lookahead_multiple),
            vec![0, 1, 2, 3, 5, 7, 9, 12, 15, 18, 22, 26, 30, 35, 40, 45, 51, 57, 60]
        );

        let lookahead_horizon = 10;
        let lookahead_multiple = 3;
        assert_eq!(
            get_variable_timesteps(lookahead_horizon, lookahead_multiple),
            vec![0, 1, 2, 3, 5, 7, 9, 10]
        );

        let lookahead_horizon = 20;
        let lookahead_multiple = 5;
        assert_eq!(
            get_variable_timesteps(lookahead_horizon, lookahead_multiple),
            vec![0, 1, 2, 3, 4, 5, 7, 9, 11, 13, 15, 18, 20],
        );
    }
}

// pub fn get_variable_timesteps(lookahead_horizon: usize, lookahead_multiple: usize) -> Vec<usize> {
//     let estimated_capacity = (2.5 * f32::sqrt(lookahead_horizon as f32)) as usize;

//     std::iter::successors(Some(0), |&x| {
//         Some(x + ((x - 1) / lookahead_multiple).saturating_add(1))
//             .filter(|&ts| ts < lookahead_horizon)
//     })
//     .take_while(|&ts| ts < lookahead_horizon)
//     .collect()
// }
