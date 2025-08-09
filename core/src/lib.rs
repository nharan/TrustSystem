#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Opinion {
    pub b: f64,
    pub d: f64,
    pub u: f64,
}

impl Opinion {
    pub fn new(b: f64, d: f64, u: f64) -> Self { Self { b, d, u } }
}

pub fn evidence_to_opinion(alpha: f64, beta: f64, prior: f64) -> Opinion {
    let denom = alpha + beta + prior;
    let b = if denom == 0.0 { 0.0 } else { alpha / denom };
    let d = if denom == 0.0 { 0.0 } else { beta / denom };
    let u = if denom == 0.0 { 1.0 } else { prior / denom };
    Opinion { b, d, u }
}

pub fn discounting(op_ab: Opinion, op_bx: Opinion) -> Opinion {
    let b = op_ab.b * op_bx.b;
    let d = op_ab.b * op_bx.d;
    let u = op_ab.d + op_ab.u + (op_ab.b * op_bx.u);
    Opinion { b, d, u }
}

pub fn consensus_fusion(o1: Opinion, o2: Opinion) -> Opinion {
    let k = o1.u + o2.u - (o1.u * o2.u);
    let b = (o1.b * o2.u + o2.b * o1.u) / k;
    let d = (o1.d * o2.u + o2.d * o1.u) / k;
    let u = (o1.u * o2.u) / k;
    Opinion { b, d, u }
}

pub fn time_decay(mut o: Opinion, delta_days: f64, half_life_days: f64) -> Opinion {
    let decay = 0.5f64.powf(delta_days / half_life_days);
    o.b *= decay;
    o.d *= decay;
    o.u = 1.0 - o.b - o.d;
    o
}

pub fn hop_decay(mut o: Opinion, lambda: f64) -> Opinion {
    o.b *= lambda;
    o.d *= lambda;
    o.u = 1.0 - o.b - o.d;
    o
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn t_evidence_to_opinion() {
        let o = evidence_to_opinion(8.0, 2.0, 2.0);
        assert_relative_eq!(o.b, 2.0/3.0, epsilon=1e-9);
        assert_relative_eq!(o.d, 1.0/6.0, epsilon=1e-9);
        assert_relative_eq!(o.u, 1.0/6.0, epsilon=1e-9);
    }

    #[test]
    fn t_discounting() {
        let ab = Opinion::new(0.7, 0.1, 0.2);
        let bx = Opinion::new(0.6, 0.2, 0.2);
        let ax = discounting(ab, bx);
        assert_relative_eq!(ax.b, 0.42, epsilon=1e-9);
        assert_relative_eq!(ax.d, 0.14, epsilon=1e-9);
        assert_relative_eq!(ax.u, 0.44, epsilon=1e-9);
    }

    #[test]
    fn t_hop_decay() {
        let o = Opinion::new(0.42, 0.14, 0.44);
        let o2 = hop_decay(o, 0.85);
        assert_relative_eq!(o2.b, 0.357, epsilon=1e-9);
        assert_relative_eq!(o2.d, 0.119, epsilon=1e-9);
        assert_relative_eq!(o2.u, 1.0 - 0.357 - 0.119, epsilon=1e-9);
    }

    #[test]
    fn t_consensus() {
        let o1 = Opinion::new(0.6, 0.2, 0.2);
        let o2 = Opinion::new(0.5, 0.3, 0.2);
        let o = consensus_fusion(o1, o2);
        assert_relative_eq!(o.b, 0.611_111_111_1, epsilon=1e-9);
        assert_relative_eq!(o.d, 0.277_777_777_8, epsilon=1e-9);
        assert_relative_eq!(o.u, 0.111_111_111_1, epsilon=1e-9);
    }

    #[test]
    fn t_time_decay() {
        let o = Opinion::new(0.6, 0.2, 0.2);
        let o2 = time_decay(o, 30.0, 30.0);
        assert_relative_eq!(o2.b, 0.3, epsilon=1e-9);
        assert_relative_eq!(o2.d, 0.1, epsilon=1e-9);
        assert_relative_eq!(o2.u, 0.6, epsilon=1e-9);
    }
}


