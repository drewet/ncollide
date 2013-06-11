use std::managed;
use broad::broad_phase::BroadPhase;

struct BruteForceBroadPhase<RB>
{
  priv objects: ~[@mut RB],
  priv panding: ~[@mut RB]
}

impl<RB> BruteForceBroadPhase<RB>
{
  pub fn new() -> BruteForceBroadPhase<RB>
  {
    BruteForceBroadPhase {
      objects: ~[],
      panding: ~[]
    }
  }
}

// FIXME: this is a workaround the fact no Eq instance is detected for the
// pointer @mut (even with the 'use std::managed').
fn position_elem_mut_ptr<RB>(l: &[@mut RB], e: @mut RB) -> Option<uint>
{
  for l.eachi |i, &curr|
  {
    if managed::mut_ptr_eq(e, curr)
    { return Some(i) }
  }

  None
}

impl<RB> BroadPhase<RB> for BruteForceBroadPhase<RB>
{
  fn add(&mut self, b: @mut RB)
  { self.panding.push(b); }

  fn remove(&mut self, b: @mut RB)
  {
    match position_elem_mut_ptr(self.objects, b)
    {
      None => {
        match position_elem_mut_ptr(self.panding, b)
        {
          None    => fail!("Tried to remove an unexisting element."),
          Some(i) => self.panding.remove(i)
        }
      },

      Some(i) => self.objects.remove(i)
    };
  }

  fn collision_pairs(&mut self, _: &[@mut RB]) -> ~[(@mut RB, @mut RB)]
  {
    let mut res: ~[(@mut RB, @mut RB)] = ~[];

    for self.panding.each |&o|
    {
      for self.objects.each |&o2|
      { res.push((o, o2)) }

      for self.panding.each |&o2|
      {
        if (managed::mut_ptr_eq(o, o2))
        { res.push((o, o2)) }
      }
    }

    for self.panding.each |&o|
    { self.objects.push(o) }

    self.panding.clear();

    res
  }
}
