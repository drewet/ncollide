use nalgebra::na::Transform;
use nalgebra::na;
use narrow::CollisionDetector;
use contact::Contact;
use math::{N, V, M};

#[deriving(Encodable, Decodable, Clone)]
struct ContactWLocals {
    local1:  V,
    local2:  V,
    center:  V,
    contact: Contact
}

impl ContactWLocals {
    fn new_with_contact(contact: Contact, m1: &M, m2: &M) -> ContactWLocals {
            ContactWLocals {
                local1: m1.inv_transform(&contact.world1),
                local2: m2.inv_transform(&contact.world2),
                center: (contact.world1 + contact.world2) * na::cast::<f32, N>(0.5),
                contact: contact
            }
        }
}

/// Contact manifold generator which keeps track of several contacts.
///
/// One contact is added per update until the maximum number of contact is reached. When the
/// maximum number of contact is reached, each time a new contact is created, the new manifold is
/// computed by maximizing the variance along each canonical axis (of the space in which leaves the
/// contacts).
#[deriving(Encodable, Decodable, Clone)]
pub struct IncrementalContactManifoldGenerator<CD> {
    priv contacts:     ~[ContactWLocals],
    priv collector:    ~[Contact],
    priv prediction:   N,
    priv sub_detector: CD
}

impl<CD> IncrementalContactManifoldGenerator<CD> {
    /// Creates a new incremental contact manifold generator.
    ///
    /// # Arguments:
    /// * `cd` - collision detection sub-algorithm used to generate the contact points.
    pub fn new(prediction: N, cd: CD) -> IncrementalContactManifoldGenerator<CD> {
        IncrementalContactManifoldGenerator {
            contacts:     ~[],
            collector:    ~[],
            prediction:   prediction,
            sub_detector: cd
        }
    }
}

impl<CD: CollisionDetector<G1, G2>, G1, G2> IncrementalContactManifoldGenerator<CD> {
    /// Gets a collision from the sub-detector used by this manifold generator. This does not
    /// update the manifold itself.
    pub fn get_sub_collision(&mut self, m1: &M, g1: &G1, m2: &M, g2: &G2) -> Option<Contact> {
        self.sub_detector.update(m1, g1, m2, g2);
        self.sub_detector.colls(&mut self.collector);

        let res = if self.collector.len() == 0 {
            None
        }
        else {
            Some(self.collector[0].clone())
        };

        self.collector.clear();

        res
    }

    /// Updates the current manifold by adding one point.
    pub fn add_new_contacts(&mut self, m1: &M, g1: &G1, m2: &M, g2: &G2) {
        // add the new ones
        self.sub_detector.update(m1, g1, m2, g2);

        self.sub_detector.colls(&mut self.collector);

        // remove duplicates
        let _max_num_contact = (na::dim::<V>() - 1) * 2;

        for c in self.collector.iter() {
            if self.contacts.len() == _max_num_contact {
                add_reduce_by_variance(self.contacts, c.clone(), m1, m2)
            }
            else {
                self.contacts.push(ContactWLocals::new_with_contact(c.clone(), m1, m2))
            }
        }

        self.collector.clear();
    }

    /// Updates the contacts already existing on this manifold.
    pub fn update_contacts(&mut self, m1: &M, m2: &M) {
        // cleanup existing contacts
        let mut i = 0;
        while i != self.contacts.len() {
            let remove = {
                let c      = &mut self.contacts[i];
                let world1 = m1.transform(&c.local1);
                let world2 = m2.transform(&c.local2);

                let dw    = world1 - world2;
                let depth = na::dot(&dw, &c.contact.normal);

                if depth >= -self.prediction &&
                   na::sqnorm(&(dw - c.contact.normal * depth)) <= na::cast(0.01) {
                        c.contact.depth  = depth;
                        c.contact.world1 = world1;
                        c.contact.world2 = world2;

                        false
                    }
                else {
                    true
                }
            };

            if remove {
                self.contacts.swap_remove(i);
            }
            else {
                i = i + 1;
            }
        }
    }
}

impl<CD: CollisionDetector<G1, G2>, G1, G2>
CollisionDetector<G1, G2> for IncrementalContactManifoldGenerator<CD> {
    #[inline]
    fn update(&mut self, m1: &M, g1: &G1, m2: &M, g2: &G2) {
        self.update_contacts(m1, m2);
        self.add_new_contacts(m1, g1, m2, g2);
    }

    #[inline]
    fn num_colls(&self) -> uint {
        self.contacts.len()
    }

    #[inline]
    fn colls(&self, out_colls: &mut ~[Contact]) {
        for c in self.contacts.iter() {
            out_colls.push(c.contact.clone())
        }
    }

    #[inline]
    fn toi(_:    Option<IncrementalContactManifoldGenerator<CD>>,
           m1:   &M,
           dir:  &V,
           dist: &N,
           g1:   &G1,
           m2:   &M,
           g2:   &G2) -> Option<N> {
        CollisionDetector::toi(None::<CD>, m1, dir, dist, g1, m2, g2)
    }

}

fn add_reduce_by_variance(pts: &mut [ContactWLocals], to_add: Contact, m1: &M, m2: &M) {
    let mut argmax = 0;
    let mut varmax = approx_variance(pts, &to_add, 0);

    for i in range(1u, pts.len()) {
        let var = approx_variance(pts, &to_add, i);

        if var > varmax {
            argmax = i;
            varmax = var;
        }
    }

    pts[argmax] = ContactWLocals::new_with_contact(to_add, m1, m2);
}

fn approx_variance(pts: &[ContactWLocals], to_add: &Contact, to_ignore: uint) -> N {
    // first: compute the mean
    let to_add_center = (to_add.world1 + to_add.world2) * na::cast::<f32, N>(0.5);

    let mut mean = to_add_center.clone();

    for i in range(0u, pts.len()) {
        if i != to_ignore {
            mean = mean + pts[i].center
        }
    }

    let divisor: f32 = 1.0 / na::cast(pts.len());
    mean = mean * na::cast::<f32, N>(divisor);

    // compute the sum of variances along all axis
    let mut sum = na::sqnorm(&(to_add_center - mean));

    for i in range(0u, pts.len()) {
        if i != to_ignore {
            sum = sum + na::sqnorm(&(pts[i].center - mean));
        }
    }

    sum
}
