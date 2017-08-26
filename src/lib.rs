/// A system for writing cross-AVR routines of code for distributed processing and fun!

macro_rules! __make_caller {

    (rt $routine_name:ident($($var_name:ident: $var_type:ty),*) -> $ret:ty $block:block $($tail:tt)*) => {
        fn $routine_name($($var_name: $var_type:ty),*) -> $ret {
            unimplemented!()
        }
        __make_caller!($($tail)*);
    };

    (pub rt $routine_name:ident($($var_name:ident: $var_type:ty),*) -> $ret:ty $block:block $($tail:tt)*) => {
        pub fn $routine_name($($var_name: $var_type:ty),*) -> $ret {
            unimplemented!()
        }
        __make_caller!($($tail)*);
    };

    () => ();
}

macro_rules! __make_callbacks {

    (rt $routine_name:ident($($var_name:ident: $var_type:ty),*) -> $ret:ty $block:block $($tail:tt)*) => {
        pub fn $routine_name($($var_name: $var_type:ty),*) -> $ret $block
        __make_callbacks!($($tail)*);
    };

    (pub rt $routine_name:ident($($var_name:ident: $var_type:ty),*) -> $ret:ty $block:block $($tail:tt)*) => {
        pub fn $routine_name($($var_name: $var_type:ty),*) -> $ret $block
        __make_callbacks!($($tail)*);
    };

    () => ();
}

macro_rules! __make_ifs {

    ($match_var:ident, rt $routine_name:ident($($var_name:ident: $var_type:ty),*) -> $ret:ty $block:block $($tail:tt)*) => {
        if $match_var == stringify!($routine_name) {
            self::callbacks::$routine_name();
            return Ok(());
        }
        __make_ifs!($match_var, $($tail)*);
    };

    ($match_var:ident, pub rt $routine_name:ident($($var_name:ident: $var_type:ty),*) -> $ret:ty $block:block $($tail:tt)*) => {
        if $match_var == stringify!($routine_name) {
            self::callbacks::$routine_name();
            return Ok(());
        }
        __make_ifs!($match_var, $($tail)*);
    };

    ($match_var:ident,) => ();
}

/// The main macro - a block for defining your routines
///
/// # Definition
///
/// Routines are defined exactly as one would define a function, with two exceptions:
///
/// 1. Routines must have a return type in their signature. It doesn't make sense for multiprocessing
/// code to return nothing. That would be pointless
///
/// 2. Routines are defined using the `rt` keyword, as opposed to `fn`
///
/// Otherwise, writing routines is just like writing functions.
///
/// # Example
///
/// ```rust
/// routines! {
///     rt private_routine -> &'static str {
///         "A private routine. Notice that a routine is defined similarly to a function"
///     }
///
///     // Error! Routines must return
///     // There is no reason to have a routine that doesn't return anything
///     // After all, routines are used for multiprocessing; returning nothing doesn't make sense
///     rt no_return_routine {
///     }
///
///     // Allowed, but strongly discouraged
///     rt no_return_routine_explicit -> () {
///     }
/// }
/// ```
///
/// # Usage
///
/// *Note: this unimplemented currently*
///
/// Routines do not return their specified return types -- instead, they return a handle to the
/// return, much like a future. You can wait on one using `await`, and check if it has completed
/// using `poll`.
///
/// ## Usage Example
///
/// ```rust
/// routines! {
///     /// Long, slow calculation
///     rt some_long_calculation {
///         // ...
///     }
/// }
///
/// fn main() {
///     let result = some_long_calculation();
///
///     if let Some(res) = result.poll() {
///         // ...
///     }
///
///     let result_value = result.await();
/// }
/// ```
///
///
/// # Caveats
///
/// Routines are *not* functions. For instance, they have no access to the global scope that is
/// *safe*, because routines are run on separate processors. It is not guaranteed that routines
/// will always run on the same processor, therefore it is discouraged to use global variables.
///
/// # Workings
///
/// *Note: this is unimplemented currently*
///
/// ## Infrastructure
///
/// This system requires a bare minimum of 3 AVRs linked up by SPI and serial respectively.
///
/// There are 3 types of processors:
///
/// 1. Main -- the main code runs here. Has a link via SPI and serial to the master
/// 2. Master -- the multiplexer, relays data to and from slaves and main. Acts as an SPI master,
/// with the slaves and main processor as slaves.
/// 3. Slave -- executes routines. Has a link via SPI to the master
///
/// ## Implementation
///
/// Calling a routine is implemented by first sending a select packet to the master controller over
/// serial. This triggers the master controller to select the main controller in SPI. The main controller
/// then sends the routine calling packet. The master selects the slave with the least work, and sends
/// the calling packet there. The slave receives the packet, and executes the routine. When the slave
/// is done, it waits until the master has selected it again, on its routine poll for completion.
/// The slave then sends the routine completed packet, and then sends the return value. It remains
/// selected for the transfer. The master then stores the return, and waits for the main to request
/// the return of that routine. When it does, the master will send the data to the main for usage.
///
/// ## Diagram
/// ![Diagram of MP4A data flow](https://i.imgur.com/M5Truwo.png)
// Thank you to L117 for helping me get optional pub
// modifier right, along with a bunch of other stuff
macro_rules! routines {
    {$($tokens:tt)*} => { // TODO args

        __make_caller!($($tokens)+);

        mod __routines {
            mod callbacks {
                __make_callbacks!($($tokens)+);
            }

            pub fn handle(name: &str/*, args: [u8]*/) -> Result<(), String> {
                __make_ifs!(name, $($tokens)+);

                Err("routine with name \"".to_owned() + name + "\" not found") // TODO use core
            }
        }
    }
}

#[cfg(test)]
mod test {
    routines! {
        rt hello() -> &'static str {
            "Hello!"
        }
    }

    #[test]
    fn test_handle() {
        if let Ok(_) = __routines::handle("hello") {
            println!("Success, ok result on existing routine");
        } else {
            panic!("handle should return ok on existing routine");
        }


        if let Err(_) = __routines::handle("nonexistent") {
            println!("Success, error result on nonexistent routine");
        } else {
            panic!("handle should return error result on nonexistent routine");
        }
    }
}
