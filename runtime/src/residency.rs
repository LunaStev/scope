//! Bounded LRU residency with frame pinning: O(log n) touches/evictions.
use std::{collections::{HashMap,BTreeSet},hash::Hash};
struct Entry<V>{value:V,bytes:usize,touched:u64}
pub struct Residency<K,V>{items:HashMap<K,Entry<V>>,order:BTreeSet<(u64,K)>,bytes:usize,pub max_bytes:usize,pub max_entries:usize}
impl<K:Copy+Ord+Hash,V> Residency<K,V>{
    pub fn new(max_bytes:usize,max_entries:usize)->Self{Self{items:HashMap::new(),order:BTreeSet::new(),bytes:0,max_bytes,max_entries}}
    pub fn len(&self)->usize{self.items.len()}
    pub fn is_empty(&self)->bool{self.items.is_empty()}
    pub fn bytes(&self)->usize{self.bytes}
    pub fn clear(&mut self){self.items.clear();self.order.clear();self.bytes=0;}
    pub fn contains(&self,key:K)->bool{self.items.contains_key(&key)}
    pub fn get(&mut self,key:K,frame:u64)->Option<&mut V>{let e=self.items.get_mut(&key)?;if e.touched!=frame{self.order.remove(&(e.touched,key));e.touched=frame;self.order.insert((frame,key));}Some(&mut e.value)}
    pub fn pin_where(&mut self,frame:u64,mut visible:impl FnMut(&K,&V)->bool){for(&key,e)in &mut self.items{if e.touched!=frame&&visible(&key,&e.value){self.order.remove(&(e.touched,key));e.touched=frame;self.order.insert((frame,key));}}}
    pub fn make_room(&mut self,bytes:usize,frame:u64)->bool{
        if bytes>self.max_bytes||self.max_entries==0{return false;}
        while self.bytes+bytes>self.max_bytes||self.len()>=self.max_entries{
            let Some(&(age,key))=self.order.first()else{return false;};if age>=frame{return false;}
            self.order.remove(&(age,key));if let Some(e)=self.items.remove(&key){self.bytes-=e.bytes;}
        }true
    }
    pub fn insert(&mut self,key:K,value:V,bytes:usize,frame:u64)->bool{if self.items.contains_key(&key)||!self.make_room(bytes,frame){return false;}self.items.insert(key,Entry{value,bytes,touched:frame});self.order.insert((frame,key));self.bytes+=bytes;true}
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn never_evict_a_current_frame(){let mut c=Residency::new(10,2);assert!(c.insert(1,"a",6,1));assert!(!c.insert(2,"b",6,1));assert!(c.insert(2,"b",6,2));assert!(!c.contains(1));assert_eq!(c.bytes(),6);}
    #[test]fn access_refreshes_recency(){let mut c=Residency::new(30,2);c.insert(1,1,10,1);c.insert(2,2,10,1);c.get(1,2);assert!(c.insert(3,3,10,2));assert!(c.contains(1));assert!(!c.contains(2));}
    #[test]fn oversize_does_not_flush_healthy_cache(){let mut c=Residency::new(10,2);c.insert(1,1,5,1);assert!(!c.insert(2,2,11,2));assert_eq!(c.len(),1);}
    #[test]fn visible_working_set_is_pinned_before_misses(){let mut c=Residency::new(20,2);c.insert(1,1,10,1);c.insert(2,2,10,1);c.pin_where(2,|_,_|true);assert!(!c.insert(3,3,10,2));assert_eq!(c.len(),2);}
}
