struct IntVar(i32);

// MACROS---------------------------------

// Call-site macros.
//
// Rust `macro_rules!` restrictions:
// - `$func:path` cannot be followed directly by `(`
// - `$recv:expr` cannot be followed directly by `.`
//
// Workarounds used below:
// - for paths, use `{ ... }` instead of `( ... )`
// - for methods, require the receiver wrapped in `( ... )` before `.`

#[macro_export]
macro_rules! own_call {
  // Usage:
  //   own_call!(out = func(a, b));
  //   own_call!(_ = func(a, b)); // ignore return
  ($out:pat = $func:ident ( $($arg:ident),* $(,)? ) ; ) => {
    let ($out, $($arg),*) = $func($($arg),*);
  };

  // Path form:
  //   own_call!(out = crate::m::func{a, b});
  ($out:pat = $func:path { $($arg:ident),* $(,)? } ; ) => {
    let ($out, $($arg),*) = $func($($arg),*);
  };

  // Zero-arg:
  ($out:pat = $func:ident () ; ) => {
    let ($out,) = $func();
  };
  ($out:pat = $func:path {} ; ) => {
    let ($out,) = $func();
  };
}

#[macro_export]
macro_rules! own_call_mut {
  // Same as own_call!, but keeps rebound vars `mut` after the call.

  ($func:ident ( $($arg:ident),* $(,)? )) => {{
    let (__ret, $(mut $arg),*) = $func($($arg),*);
    __ret
  }};

  ($func:path { $($arg:ident),* $(,)? }) => {{
    let (__ret, $(mut $arg),*) = $func($($arg),*);
    __ret
  }};

  ( ($recv:expr) . $method:ident ( $($arg:ident),* $(,)? ) ) => {{
    let (__ret, $(mut $arg),*) = ($recv).$method($($arg),*);
    __ret
  }};

  ($func:ident ()) => {{
    let (__ret,) = $func();
    __ret
  }};
  ($func:path {}) => {{
    let (__ret,) = $func();
    __ret
  }};
  ( ($recv:expr) . $method:ident () ) => {{
    let (__ret,) = ($recv).$method();
    __ret
  }};
}

// Fn-site macro: defines a function that returns `(ret, args...)`.
//
// Restriction (macro_rules!): `return expr;` inside the body is not rewritten.
// If you need early returns, switch this to a proc-macro attribute.
// Fix: avoid `$(mut)? $arg:ident` ambiguity by using a TT-muncher that
// matches `mut` explicitly, and extracts the *identifier list* + *type list*.
//
// Behavior:
// - You may write args as `x: T` or `mut x: T` (either works).
// - The generated function signature does NOT require `mut` on params.
//   Instead, it inserts `let mut x = x;` at the top so the body can mutate.
// - Returns `(ret, args...)` or `((), args...)` if no return type is specified.
// - Early `return expr;` is NOT rewritten (macro_rules! limitation).

#[macro_export]
macro_rules! own_fn {
  // Explicit return type
  (fn $name:ident ( $($args:tt)* ) -> $ret:ty $body:block) => {
    $crate::own_fn!(@munch_ret
      fn $name
      ( /* idents */ )
      ( /* tys    */ )
      ( $($args)* )
      -> $ret
      $body
    );
  };

  // No return type (defaults to `()`)
  (fn $name:ident ( $($args:tt)* ) $body:block) => {
    $crate::own_fn!(@munch_unit
      fn $name
      ( /* idents */ )
      ( /* tys    */ )
      ( $($args)* )
      $body
    );
  };

  // -----------------------
  // Muncher for -> $ret
  // -----------------------

  // End (no more tokens)
  (@munch_ret fn $name:ident ( $($id:ident,)* ) ( $($ty:ty,)* ) ( ) -> $ret:ty $body:block) => {
    fn $name( $( $id : $ty ),* ) -> ( $ret, $( $ty ),* ) {
      $( let mut $id = $id; )*
      let __ret: $ret = (|| $body)();
      ( __ret, $( $id ),* )
    }
  };

  // mut arg with trailing comma
  (@munch_ret fn $name:ident ( $($id:ident,)* ) ( $($ty:ty,)* )
    ( mut $arg:ident : $t:ty , $($rest:tt)* ) -> $ret:ty $body:block
  ) => {
    $crate::own_fn!(@munch_ret fn $name ( $($id,)* $arg, ) ( $($ty,)* $t, ) ( $($rest)* ) -> $ret $body);
  };

  // non-mut arg with trailing comma
  (@munch_ret fn $name:ident ( $($id:ident,)* ) ( $($ty:ty,)* )
    ( $arg:ident : $t:ty , $($rest:tt)* ) -> $ret:ty $body:block
  ) => {
    $crate::own_fn!(@munch_ret fn $name ( $($id,)* $arg, ) ( $($ty,)* $t, ) ( $($rest)* ) -> $ret $body);
  };

  // mut arg last (no comma)
  (@munch_ret fn $name:ident ( $($id:ident,)* ) ( $($ty:ty,)* )
    ( mut $arg:ident : $t:ty ) -> $ret:ty $body:block
  ) => {
    $crate::own_fn!(@munch_ret fn $name ( $($id,)* $arg, ) ( $($ty,)* $t, ) ( ) -> $ret $body);
  };

  // non-mut arg last (no comma)
  (@munch_ret fn $name:ident ( $($id:ident,)* ) ( $($ty:ty,)* )
    ( $arg:ident : $t:ty ) -> $ret:ty $body:block
  ) => {
    $crate::own_fn!(@munch_ret fn $name ( $($id,)* $arg, ) ( $($ty,)* $t, ) ( ) -> $ret $body);
  };

  // trailing comma only
  (@munch_ret fn $name:ident ( $($id:ident,)* ) ( $($ty:ty,)* )
    ( , ) -> $ret:ty $body:block
  ) => {
    $crate::own_fn!(@munch_ret fn $name ( $($id,)* ) ( $($ty,)* ) ( ) -> $ret $body);
  };

  // -----------------------
  // Muncher for unit return
  // -----------------------

  (@munch_unit fn $name:ident ( $($id:ident,)* ) ( $($ty:ty,)* ) ( ) $body:block) => {
    fn $name( $( $id : $ty ),* ) -> ( (), $( $ty ),* ) {
      $( let mut $id = $id; )*
      (|| $body)();
      ( (), $( $id ),* )
    }
  };

  (@munch_unit fn $name:ident ( $($id:ident,)* ) ( $($ty:ty,)* )
    ( mut $arg:ident : $t:ty , $($rest:tt)* ) $body:block
  ) => {
    $crate::own_fn!(@munch_unit fn $name ( $($id,)* $arg, ) ( $($ty,)* $t, ) ( $($rest)* ) $body);
  };

  (@munch_unit fn $name:ident ( $($id:ident,)* ) ( $($ty:ty,)* )
    ( $arg:ident : $t:ty , $($rest:tt)* ) $body:block
  ) => {
    $crate::own_fn!(@munch_unit fn $name ( $($id,)* $arg, ) ( $($ty,)* $t, ) ( $($rest)* ) $body);
  };

  (@munch_unit fn $name:ident ( $($id:ident,)* ) ( $($ty:ty,)* )
    ( mut $arg:ident : $t:ty ) $body:block
  ) => {
    $crate::own_fn!(@munch_unit fn $name ( $($id,)* $arg, ) ( $($ty,)* $t, ) ( ) $body);
  };

  (@munch_unit fn $name:ident ( $($id:ident,)* ) ( $($ty:ty,)* )
    ( $arg:ident : $t:ty ) $body:block
  ) => {
    $crate::own_fn!(@munch_unit fn $name ( $($id,)* $arg, ) ( $($ty,)* $t, ) ( ) $body);
  };

  (@munch_unit fn $name:ident ( $($id:ident,)* ) ( $($ty:ty,)* )
    ( , ) $body:block
  ) => {
    $crate::own_fn!(@munch_unit fn $name ( $($id,)* ) ( $($ty,)* ) ( ) $body);
  };
}

// CODE---------------------------------

impl std::fmt::Display for IntVar {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "{}", self.0)
    }
}

impl From<i32> for IntVar{
    fn from(x: i32) -> Self {
        IntVar(x)
    }
}

impl From<IntVar> for i32 {
    fn from(x: IntVar) -> Self {
        x.0
    }
}


fn do_something(x: &mut IntVar) {
    x.0 = x.0 + 1;
    println!("inside func: {}", x.0);
}

fn add_simple_ref(x: &mut i32) {
    *x = *x + 1;
    println!("inside func: {}", x);
}

fn add_simple(mut x: i32) {
    x = x + 1;
    println!("inside func: {}", x);
}

fn modify_vec(x: Vec<i32>) -> Vec<i32> {
    x
}

own_fn! {
  fn macro_vec(mut x: Vec<i32>) {
    for val in x.iter_mut() {
      *val *= 100;
    }
  }
}


fn main() {
    // let mut x = IntVar(10);
    // do_something(&mut x);
    
    // let x = 10;
    // add_simple(x);
    
    // let mut x = 10;
    // add_simple(&mut x);

    let x = vec![1, 2, 3];
    // own_call!( macro_vec(x) );
    own_call!(_ = macro_vec(x););

    println!("{:?}", x);
}
