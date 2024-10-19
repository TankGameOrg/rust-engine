use std::{any::{Any, TypeId}, collections::HashMap, fmt::Display, marker::PhantomData};
use core::error::Error;

#[derive(Debug)]
struct BasicError(String);

impl Display for BasicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for BasicError {}

#[derive(Hash, Eq, Debug)]
struct Attribute<T> {
    name: &'static str,
    phantom: PhantomData<T>,
}

impl<T> PartialEq for Attribute<T> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<T> Attribute<T> {
    const fn new(name: &'static str) -> Attribute<T> {
        Attribute {
            name,
            phantom: PhantomData,
        }
    }
}

static DURABILITY: Attribute<u32> = Attribute::new("durability");
static PLAYER: Attribute<Handle> = Attribute::new("player");
static DAILY_DAMAGE: Attribute<u32> = Attribute::new("daily_damage");

#[derive(Debug)]
struct AttributeContainer {
    attributes: HashMap<&'static str, Box<dyn Any>>
}

impl AttributeContainer {
    fn new() -> AttributeContainer {
        AttributeContainer {
            attributes: HashMap::new(),
        }
    }

    fn get<T: 'static>(&self, key: &Attribute<T>) -> Result<&T, Box<dyn Error>> {
        match self.attributes.get(key.name) {
            Some(any) => {
                match any.downcast_ref::<T>() {
                    Some(value) => Ok(value),
                    None => Err(Box::new(BasicError(String::from("Wrong type for unwrap")))),
                }
            },
            None => Err(Box::new(BasicError(format!("Attribute '{}' does not exist", key.name)))),
        }
    }

    fn put<T: 'static>(&mut self, key: &Attribute<T>, value: T) {
        self.attributes.insert(key.name, Box::new(value));
    }

    fn has<T: 'static>(&self, key: &Attribute<T>) -> bool {
        self.attributes.contains_key(key.name)
    }

    fn visit_all(&self, visitor: &dyn Fn(&'static str, &dyn Any) -> Result<(), Box<dyn Error>>) -> Result<(), Box<dyn Error>> {
        for (attribute, value) in &self.attributes {
            (visitor)(attribute, value.as_ref())?;
        }

        Ok(())
    }
}

macro_rules! make_visitor {
    ($(($var_attribute_name:ident, $var_name:ident: $attribute_type:ty) => $code:expr),+) => {
        |attribute_name: &'static str, any_attribute_value: &dyn Any| -> Result<(), Box<dyn Error>> {
            let attribute_type_id = any_attribute_value.type_id();
            let mut attribute_value_handled = false;

            $(
                if attribute_type_id == TypeId::of::<$attribute_type>() {
                    let $var_attribute_name = attribute_name;
                    attribute_value_handled = true;

                    match any_attribute_value.downcast_ref::<$attribute_type>() {
                        Some($var_name) => $code,
                        None => panic!("Not allowed"),
                    }
                }
            )+

            if !attribute_value_handled {
                Err(Box::new(BasicError(format!("No case found for type: {:?}", attribute_type_id))))
            } else {
                Ok(())
            }
        }
    };
}

trait Modification {
    fn apply(&self, pool: &mut Pool) -> Result<(), Box<dyn Error>>;
}

struct AttributeModification<T: 'static + Clone> {
    handle: Handle,
    attribute: &'static Attribute<T>,
    new_value: T
}

impl<T: 'static + Clone> AttributeModification<T> {
    fn new(handle: Handle, attribute: &'static Attribute<T>, new_value: T) -> AttributeModification<T> {
        AttributeModification {
            handle,
            attribute,
            new_value,
        }
    }
}

impl<T: 'static + Clone> Modification for AttributeModification<T> {
    fn apply(&self, pool: &mut Pool) -> Result<(), Box<dyn Error>> {
        pool.get_attribute_container_mut(self.handle)?.put(self.attribute, self.new_value.clone());
        Ok(())
    }
}

struct AddContainerModification {
    handle: Handle,
}

impl AddContainerModification {
    fn new() -> (Handle, AddContainerModification) {
        let handle = Handle::new();
        (handle, AddContainerModification {
            handle,
        })
    }
}

impl Modification for AddContainerModification {
    fn apply(&self, pool: &mut Pool) -> Result<(), Box<dyn Error>> {
        pool.add_attribute_container(self.handle, AttributeContainer::new());
        Ok(())
    }
}

struct Transaction {
    modifications: Vec<Box<dyn Modification>>
}

impl Transaction {
    fn new() -> Transaction {
        Transaction {
            modifications: Vec::new(),
        }
    }

    fn add<T: Modification + 'static>(&mut self, modification: T) {
        self.modifications.push(Box::new(modification));
    }

    fn apply(self, pool: &mut Pool) -> Result<(), Box<dyn Error>> {
        for modification in self.modifications {
            modification.apply(pool)?;
        }

        Ok(())
    }
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
struct Handle {
    handle: u32,
}

static mut NEXT_HANDLE: u32 = 0; // I was too lazy to find the Uuid type stop judging me

impl Handle {
    fn new() -> Handle {
        Handle {
            handle: unsafe { NEXT_HANDLE += 1; NEXT_HANDLE },
        }
    }
}

#[derive(Debug)]
struct Pool {
    containers: HashMap<Handle, AttributeContainer>
}

impl Pool {
    fn new() -> Pool {
        Pool {
            containers: HashMap::new(),
        }
    }

    fn add_attribute_container(&mut self, handle: Handle, container: AttributeContainer) {
        self.containers.insert(handle, container);
    }

    fn get_attribute_container(&self, handle: Handle) -> Result<&AttributeContainer, Box<dyn Error>> {
        self.containers.get(&handle)
            .ok_or(Box::new(BasicError(format!("Attribute container for {:?} does not exist", handle))))
    }

    fn get_attribute_container_mut(&mut self, handle: Handle) -> Result<&mut AttributeContainer, Box<dyn Error>> {
        self.containers.get_mut(&handle)
            .ok_or(Box::new(BasicError(format!("Attribute container for {:?} does not exist", handle))))
    }

    fn gather<'a>(&'a self, predicate: &'a dyn Fn(&AttributeContainer) -> bool) -> impl Iterator<Item = (Handle, &'a AttributeContainer)> {
        self.containers.iter()
            .filter(|(_, container)| predicate(*container))
            .map(|(handle, container)| (*handle, container))
    }
}

fn damage_all(pool: &mut Pool) -> Result<Transaction, Box<dyn Error>> {
    let mut transaction = Transaction::new();

    for (handle, target) in pool.gather(&|container| container.has(&DAILY_DAMAGE)) {
        let new_durability = target.get(&DURABILITY)? - target.get(&DAILY_DAMAGE)?;
        transaction.add(AttributeModification::new(handle, &DURABILITY, new_durability));
    }

    Ok(transaction)
}

fn do_things(pool: &mut Pool, tank_handle: Handle) -> Result<Transaction, Box<dyn Error>> {
    let tank = pool.get_attribute_container(tank_handle)?;

    let mut transaction = Transaction::new();

    let (player_handle, add_container_modification) = AddContainerModification::new();
    transaction.add(add_container_modification);
    transaction.add(AttributeModification::new(player_handle, &DURABILITY, 1));

    transaction.add(AttributeModification::new(tank_handle, &DURABILITY, tank.get(&DURABILITY)? + 1));
    transaction.add(AttributeModification::new(tank_handle, &PLAYER, player_handle));
    transaction.add(AttributeModification::new(tank_handle, &DAILY_DAMAGE, 1));

    Ok(transaction)
}

fn dump(pool: &Pool, tank_handle: Handle) -> Result<(), Box<dyn Error>> {
    let tank= pool.get_attribute_container(tank_handle)?;
    println!("\n===== Tank =====");
    dump_ctr(pool, tank)?;

    println!("\n===== Player =====");
    let player_handle = *tank.get(&PLAYER)?;
    let player = pool.get_attribute_container(player_handle)?;
    dump_ctr(pool, player)?;

    Ok(())
}

fn dump_ctr(pool: &Pool, attribute_container: &AttributeContainer) -> Result<(), Box<dyn Error>> {
    attribute_container.visit_all(&make_visitor! {
        (name, num: u32) => println!("{} = {}", name, num),
        (name, handle: Handle) => {
            match pool.get_attribute_container(*handle) {
                Ok(container) => {
                    println!("{} = [", name);
                    dump_ctr(pool, container)?;
                    println!("]");
                },
                Err(_) => {
                    println!("{} = Error: Failed to get handle {:?}", name, handle);
                }
            }

            println!("{} = {:?}", name, handle);
        }
    })
}

fn run_code() -> Result<(), Box<dyn Error>> {
    let mut pool = Pool::new();

    let mut init_transaction = Transaction::new();
    let (tank_handle, new_tank_modification) = AddContainerModification::new();
    init_transaction.add(new_tank_modification);
    init_transaction.add(AttributeModification::new(tank_handle, &DURABILITY, 1));
    init_transaction.apply(&mut pool)?;

    let transaction = do_things(&mut pool, tank_handle)?;
    transaction.apply(&mut pool)?;

    dump(&mut pool, tank_handle)?;
    println!("----------------------------");

    damage_all(&mut pool)?.apply(&mut pool)?;

    dump(&mut pool, tank_handle)?;

    Ok(())
}

fn main() {
    run_code().unwrap();
}
