pub mod rules;

use core::error::Error;

use rules::infrastructure::ecs::{container::AttributeContainer, pool::{Handle, Pool}, transaction::Transaction};

attribute!(DURABILITY: u32);
attribute!(PLAYER: Handle);
attribute!(DAILY_DAMAGE: u32);

fn damage_all(pool: &mut Pool) -> Result<Transaction, Box<dyn Error>> {
    let mut transaction = Transaction::new();

    for (handle, target) in pool.gather(&|container| container.has(&DAILY_DAMAGE)) {
        let new_durability = target.get(&DURABILITY)? - target.get(&DAILY_DAMAGE)?;
        modify_container!(&mut transaction, handle, {
            DURABILITY = new_durability
        });
    }

    Ok(transaction)
}

fn do_things(pool: &mut Pool, tank_handle: Handle) -> Result<Transaction, Box<dyn Error>> {
    let tank = pool.get_attribute_container(tank_handle)?;

    let mut transaction = Transaction::new();

    let player_handle = create_container!(&mut transaction, {
        DURABILITY = 1
    });

    modify_container!(&mut transaction, tank_handle, {
        DURABILITY = tank.get(&DURABILITY)? + 1,
        PLAYER = player_handle,
        DAILY_DAMAGE = 1
    });

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
    attribute_container.visit_all(&|name, attribute_value| {
        match_type!(attribute_value, {
            num: u32 => println!("{} = {}", name, num),
            handle: Handle => {
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
    })
}

fn run_code() -> Result<(), Box<dyn Error>> {
    let mut pool = Pool::new();

    let mut init_transaction = Transaction::new();
    let tank_handle = create_container!(&mut init_transaction, {
        DURABILITY = 1
    });

    init_transaction.apply(&mut pool)?;

    do_things(&mut pool, tank_handle)?.apply(&mut pool)?;

    dump(&mut pool, tank_handle)?;
    println!("----------------------------");

    damage_all(&mut pool)?.apply(&mut pool)?;

    dump(&mut pool, tank_handle)?;

    Ok(())
}

fn main() {
    run_code().unwrap();
}
