import init, { new_state, add_container, set_attribute, dump } from "./pkg/attributes.js";

(async () => {
    await init();
    const state = new_state();
    const container = add_container(state);
    set_attribute(container, "durability", 5);
    dump(state);
})();
