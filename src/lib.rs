// use wasm_bindgen::prelude::*;
// use std::{any::Any, collections::HashMap, fmt::Display, marker::PhantomData};
// use core::error::Error;

// #[derive(Debug)]
// struct BasicError(String);

// impl Display for BasicError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.write_str(&self.0)
//     }
// }

// impl Error for BasicError {}

// #[derive(Hash, Eq, Debug)]
// struct Attribute<T> {
//     name: &'static str,
//     phantom: PhantomData<T>,
// }

// impl<T> PartialEq for Attribute<T> {
//     fn eq(&self, other: &Self) -> bool {
//         self.name == other.name
//     }
// }

// impl<T> Attribute<T> {
//     const fn new(name: &'static str) -> Attribute<T> {
//         Attribute {
//             name,
//             phantom: PhantomData,
//         }
//     }
// }

// static DURABILITY: Attribute<u32> = Attribute::new("durability");
// static PLAYER: Attribute<Handle> = Attribute::new("player");

// #[derive(Debug)]
// struct AttributeContainer {
//     attributes: HashMap<&'static str, Box<dyn Any>>
// }

// impl AttributeContainer {
//     fn new() -> AttributeContainer {
//         AttributeContainer {
//             attributes: HashMap::new(),
//         }
//     }

//     fn get<T: 'static>(&self, key: &Attribute<T>) -> Result<&T, Box<dyn Error>> {
//         match self.attributes.get(key.name) {
//             Some(any) => {
//                 match any.downcast_ref::<T>() {
//                     Some(value) => Ok(value),
//                     None => Err(Box::new(BasicError(String::from("Wrong type for unwrap")))),
//                 }
//             },
//             None => Err(Box::new(BasicError(format!("Attribute '{}' does not exist", key.name)))),
//         }
//     }

//     fn put<T: 'static>(&mut self, key: &Attribute<T>, value: T) {
//         self.attributes.insert(key.name, Box::new(value));
//     }
// }

// trait Modification {
//     fn apply(self: Box<Self>, pool: &mut Pool) -> Result<(), Box<dyn Error>>;
// }

// struct AttributeModification<T: 'static> {
//     handle: Handle,
//     attribute: &'static Attribute<T>,
//     new_value: T
// }

// impl<T: 'static> Modification for AttributeModification<T> {
//     fn apply(self: Box<Self>, pool: &mut Pool) -> Result<(), Box<dyn Error>> {
//         pool.get_attribute_container_mut(self.handle)?.put(self.attribute, self.new_value);
//         Ok(())
//     }
// }

// struct AddContainerModification {
//     handle: Handle,
//     container: AttributeContainer,
// }

// impl Modification for AddContainerModification {
//     fn apply(self: Box<Self>, pool: &mut Pool) -> Result<(), Box<dyn Error>> {
//         pool.add_attribute_container(self.handle, self.container);
//         Ok(())
//     }
// }

// struct TransactionContainer<'transaction> {
//     handle: Handle,
//     transaction: &'transaction mut Transaction,
// }

// impl<'transaction> TransactionContainer<'transaction> {
//     fn set_attribute<T>(&mut self, attribute: &'static Attribute<T>, new_value: T) -> &mut Self {
//         self.transaction.add_modification(Box::new(AttributeModification {
//             handle: self.handle,
//             attribute,
//             new_value,
//         }));

//         self
//     }
// }

// struct Transaction {
//     modifications: Vec<Box<dyn Modification>>
// }

// impl Transaction {
//     fn new() -> Transaction {
//         Transaction {
//             modifications: Vec::new(),
//         }
//     }

//     fn add_container(&mut self) -> (Handle, TransactionContainer) {
//         let handle = Handle::new();
//         self.add_modification(Box::new(AddContainerModification {
//             handle,
//             container: AttributeContainer::new(),
//         }));

//         (handle, self.container(handle))
//     }

//     fn container(&mut self, handle: Handle) -> TransactionContainer {
//         TransactionContainer {
//             handle,
//             transaction: self,
//         }
//     }

//     fn add_modification(&mut self, modification: Box<dyn Modification>) {
//         self.modifications.push(modification);
//     }

//     fn apply(self, pool: &mut Pool) -> Result<(), Box<dyn Error>> {
//         for modification in self.modifications {
//             modification.apply(pool)?;
//         }

//         Ok(())
//     }
// }

// #[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
// struct Handle {
//     handle: u32,
// }

// static mut NEXT_HANDLE: u32 = 0; // I was too lazy to find the Uuid type stop judging me

// impl Handle {
//     fn new() -> Handle {
//         Handle {
//             handle: unsafe { NEXT_HANDLE += 1; NEXT_HANDLE },
//         }
//     }
// }

// #[derive(Debug)]
// struct Pool {
//     containers: HashMap<Handle, AttributeContainer>
// }

// impl Pool {
//     fn new() -> Pool {
//         Pool {
//             containers: HashMap::new(),
//         }
//     }

//     fn add_attribute_container(&mut self, handle: Handle, container: AttributeContainer) {
//         self.containers.insert(handle, container);
//     }

//     fn get_attribute_container(&self, handle: Handle) -> Result<&AttributeContainer, Box<dyn Error>> {
//         self.containers.get(&handle)
//             .ok_or(Box::new(BasicError(format!("Attribute container for {:?} does not exist", handle))))
//     }

//     fn get_attribute_container_mut(&mut self, handle: Handle) -> Result<&mut AttributeContainer, Box<dyn Error>> {
//         self.containers.get_mut(&handle)
//             .ok_or(Box::new(BasicError(format!("Attribute container for {:?} does not exist", handle))))
//     }
// }

// fn do_things(pool: &mut Pool, tank_handle: Handle) -> Result<Transaction, Box<dyn Error>> {
//     let tank = pool.get_attribute_container(tank_handle)?;

//     let mut transaction = Transaction::new();

//     let (player_handle, mut player_builder) = transaction.add_container();
//     player_builder.set_attribute(&DURABILITY, 1);

//     transaction.container(tank_handle)
//         .set_attribute(&DURABILITY, tank.get(&DURABILITY)? + 1)
//         .set_attribute(&PLAYER, player_handle);

//     Ok(transaction)
// }

// fn run_code() -> Result<(), Box<dyn Error>> {
//     let mut pool = Pool::new();

//     let mut init_transaction = Transaction::new();
//     let (tank_handle, mut tank_builder) = init_transaction.add_container();
//     tank_builder.set_attribute(&DURABILITY, 1);
//     init_transaction.apply(&mut pool)?;

//     let transaction = do_things(&mut pool, tank_handle)?;
//     transaction.apply(&mut pool)?;

//     let tank= pool.get_attribute_container(tank_handle)?;
//     log(format!("Tank: durability = {}", tank.get(&DURABILITY)?).as_str());

//     let player_handle = *tank.get(&PLAYER)?;
//     let player = pool.get_attribute_container(player_handle)?;
//     log(format!("Player: durability = {}", player.get(&DURABILITY)?).as_str());

//     Ok(())
// }

// fn main() {
//     run_code().unwrap();
// }

// #[wasm_bindgen]
// extern "C" {
//     // Use `js_namespace` here to bind `console.log(..)` instead of just
//     // `log(..)`
//     #[wasm_bindgen(js_namespace = console)]
//     fn log(s: &str);
// }

// #[wasm_bindgen]
// pub fn execute() {
//     run_code().unwrap();
// }

// static mut POOLS: Vec<Pool> = Vec::new();

// #[wasm_bindgen]
// #[derive(Clone, Copy)]
// pub struct PoolHandle {
//     index: usize,
// }

// impl PoolHandle {
//     fn get_state(&self) -> &'static mut Pool {
//         unsafe {
//             POOLS.get_mut(self.index).unwrap()
//         }
//     }
// }

// #[wasm_bindgen]
// pub fn new_state() -> PoolHandle {
//     unsafe {
//         POOLS.push(Pool::new());

//         PoolHandle {
//             index: POOLS.len() - 1,
//         }
//     }
// }

// #[wasm_bindgen]
// pub struct ContainerHandle {
//     pool_handle: PoolHandle,
//     handle: Handle,
// }

// #[wasm_bindgen]
// pub fn add_container(state_handle: &PoolHandle) -> ContainerHandle {
//     let pool = state_handle.get_state();

//     let mut transaction = Transaction::new();
//     let (container_handle, _) = transaction.add_container();
//     transaction.apply(pool).unwrap();

//     ContainerHandle {
//         pool_handle: *state_handle,
//         handle: container_handle,
//     }
// }

// #[wasm_bindgen]
// pub fn set_attribute(container_handle: &ContainerHandle, name: &str, value: JsValue) {
//     let pool = container_handle.pool_handle.get_state();

//     let mut transaction = Transaction::new();
//     let mut container_builder = transaction.container(container_handle.handle);

//     match name {
//         "durability" => {
//             container_builder.set_attribute(&DURABILITY, value.as_f64().unwrap() as u32);
//         },
//         name => {
//             panic!("Bad name {}", name);
//         }
//     };

//     transaction.apply(pool).unwrap();
// }

// #[wasm_bindgen]
// pub fn dump(pool_handle: &PoolHandle) {
//     log(format!("Pool: {:?}", pool_handle.get_state()).as_str());
// }

fn call_handler(handler: &dyn Fn(u32) -> u32) {
    handler(2);
}

mod test {
    #[test]
    fn foo() {

    }
}