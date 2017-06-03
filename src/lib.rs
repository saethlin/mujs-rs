//! # Rust MuJS bindings
//!
//! [MuJS](http://mujs.com) is a lightweight implementation of the
//! Javascript language in a library. MuJS is licensed under AGPL and
//! so is this rustc bindings.
//!
//! Its primary purpose and design is for embedding in other software
//! to add scripting capability to those programs, but it can also be
//! used as an extensible scripting language.
//!
//! In contrast to other programs that are large and complex, MuJS was
//! designed with a focus on small size, correctness and
//! simplicity. MuJS is written in portable C and implements
//! ECMAScript as specified by ECMA-262.
//!
//! The interface for binding with native code is designed to be as
//! simple as possible to use, and is similar to Lua.
//!

#[macro_use]
extern crate bitflags;
extern crate libc;


#[link(name = "mujs", kind="static")]
use std::ffi::{CStr, CString};


use libc::{
    c_int,
    c_double,
    c_void,
    c_char
};

extern {
    fn js_newstate(alloc: *const c_void, context: *const c_void, flags: c_int) -> *const c_void;
    fn js_freestate(J: *const c_void);
    fn js_atpanic(J: *const c_void, panic: Option<extern fn(J: *const c_void)>) -> *const c_void;
    fn js_gc(J: *const c_void, report: c_int);
    fn js_ploadstring(J: *const c_void, filename: *const c_char, source: *const c_char) -> c_int;
    fn js_pcall(J: *const c_void, n: c_int) -> c_int;
    fn js_pconstruct(J: *const c_void, n: c_int) -> c_int;
    fn js_call(J: *const c_void, n: c_int) -> c_int;
    fn js_dostring(J: *const c_void, source: *const c_char) -> c_int;

    fn js_newobject(J: *const c_void);

    fn js_isobject(J: *const c_void, idx: c_int) -> c_int;

    fn js_hasproperty(J: *const c_void, idx: c_int, name: *const c_char) -> c_int;
    fn js_getproperty(J: *const c_void, idx: c_int, name: *const c_char);
    fn js_setproperty(J: *const c_void, idx: c_int, name: *const c_char);
    fn js_defproperty(J: *const c_void, idx: c_int, name: *const c_char, attrs: c_int);
    fn js_defaccessor(J: *const c_void, idx: c_int, name: *const c_char, attrs: c_int);
    fn js_delproperty(J: *const c_void, idx: c_int, name: *const c_char);

    fn js_pushglobal(J: *const c_void);
    fn js_getglobal(J: *const c_void, name: *const c_char);
    fn js_setglobal(J: *const c_void, name: *const c_char);
    fn js_defglobal(J: *const c_void, name: *const c_char, attrs: c_int);

    fn js_pushundefined(J: *const c_void);
    fn js_pushnull(J: *const c_void);
    fn js_pushboolean(J: *const c_void, v: c_int);
    fn js_pushnumber(J: *const c_void, v: c_double);
    fn js_pushstring(J: *const c_void, v: *const c_char);

    fn js_isdefined(J: *const c_void, idx: c_int) -> c_int;
    fn js_isundefined(J: *const c_void, idx: c_int) -> c_int;

    fn js_throw(J: *const c_void);

    fn js_newerror(J: *const c_void, message: *const c_char);
    fn js_newevalerror(J: *const c_void, message: *const c_char);
    fn js_newrangeerror(J: *const c_void, message: *const c_char);
    fn js_newreferenceerror(J: *const c_void, message: *const c_char);
    fn js_newsyntaxerror(J: *const c_void, message: *const c_char);
    fn js_newtypeerror(J: *const c_void, message: *const c_char);
    fn js_newurierror(J: *const c_void, message: *const c_char);

    fn js_gettop(J: *const c_void) -> c_int;
    fn js_pop(J: *const c_void, n: c_int);
    fn js_copy(J: *const c_void, idx: c_int);

    fn js_tostring(J: *const c_void, idx: i32) -> *const c_char;
    fn js_toboolean(J: *const c_void, idx: i32) -> c_int;
    fn js_tonumber(J: *const c_void, idx: i32) -> c_double;
}

bitflags! {
    pub struct PropertyAttributes: c_int {
        /// Read only property attribute
        ///
        /// # Examples
        ///
        /// ```
        /// use mujs;
        ///
        /// let state = mujs::State::new(mujs::JS_STRICT);
        ///
        /// state.newobject();
        /// state.pushnumber(32.0);
        /// state.setproperty(-2, "age");
        /// state.defglobal("me", mujs::JS_READONLY);
        /// ```
        ///
        /// The above example defines an object in global space named
        /// ```me```, which can not be assigned another object due to the
        /// read only flag.
        const JS_READONLY = 1;
        const JS_DONTENUM = 2;
        const JS_DONTCONF = 4;
    }
}

bitflags! {
    pub struct StateFlags: c_int {
        /// Compile and run code using ES5 strict mode.
        const JS_STRICT = 1;
    }
}

/// Interpreter state contains the value stack, protected environments
/// and environment records.
pub struct State {
    state: *const c_void,
    memctx: *const c_void,
}

impl State {

    /// Constructs a new State.
    ///
    /// # Examples
    /// ```
    /// use mujs;
    ///
    /// let state = mujs::State::new(mujs::JS_STRICT);
    /// ```
    pub fn new(flags: StateFlags) -> State {
        let mut js = State {
            state: std::ptr::null(),
            memctx: std::ptr::null(),
        };

        let js_ptr: *const State = &js;
        js.memctx = js_ptr as *const c_void;
        js.state = unsafe { js_newstate(std::ptr::null(), js.memctx, flags.bits) };

        unsafe { js_atpanic(js.state, Some(State::_panic)) };

        js
    }

    extern fn _panic(J: *const c_void) {
        let top = unsafe { js_gettop(J) };
        let res_c_str = unsafe { js_tostring(J, top - 1) };
        let err = unsafe { CStr::from_ptr(res_c_str).to_string_lossy().into_owned() };
        panic!("{:?}", err);
    }

    /// Run garbage collector.
    ///
    /// # Arguments
    ///
    /// * `report` - Boolean to control report output of GC statistics to stdout
    ///
    /// # Examples
    ///
    /// ```
    /// use mujs;
    ///
    /// let state = mujs::State::new(mujs::JS_STRICT);
    ///
    /// state.gc(true);
    /// ```
    ///
    pub fn gc(self: &State, report: bool) {
        match report {
            true => unsafe { js_gc(self.state, 1) },
            false => unsafe { js_gc(self.state, 0) }
        }
    }

    /// Compile a script with result push on top of stack as a
    /// function. This function can then be executed using
    /// call() method.
    ///
    /// # Arguments
    ///
    /// * `filename` - A virtual filename for the source
    /// * `source` - A string slice with source to compile
    ///
    /// # Examples
    ///
    /// ```
    /// use mujs;
    ///
    /// let state = mujs::State::new(mujs::JS_STRICT);
    /// let source = "Math.sin(1.234)";
    ///
    /// state.loadstring("myscript", source).unwrap();
    /// state.newobject();
    /// state.call(0).unwrap();
    ///
    /// println!("{:?}", state.tostring(0).unwrap());
    /// ```
    ///
    pub fn loadstring(self: &State, filename: &str, source: &str) -> Result<(), String> {
        let name_c_str = CString::new(filename).unwrap();
        let source_c_str = CString::new(source).unwrap();
        match unsafe { js_ploadstring(self.state, name_c_str.as_ptr(), source_c_str.as_ptr()) } {
            0 => Ok(()),
            _ => {
                let err = self.tostring(-1);
                assert!(err.is_ok());
                Err(err.ok().unwrap())
            }
        }
    }

    /// Call a function pushed on stack
    ///
    /// Pop the function, this value and all arguments then executes
    /// the function. The return value is then pushed onto the stack.
    ///
    /// Follow these steps to perform a function call:
    ///
    /// 1. push the function to call onto the stack
    ///
    /// 2. push this value to be used by the function
    ///
    /// 3. push the arguments to the function in order
    ///
    /// 4. call State::call() with the numbers of arguments pushed
    ///    onto the stack
    ///
    /// # Examples
    ///
    /// ```
    /// use mujs;
    ///
    /// let state = mujs::State::new(mujs::JS_STRICT);
    ///
    /// assert!(state.loadstring("myscript", "Math.sin(3.2);").is_ok());
    /// state.newobject();
    /// assert!(state.call(0).is_ok());
    ///
    /// println!("Sin(3.2) = {:?}", state.tonumber(0).unwrap());
    ///
    /// ```
    ///
    pub fn call(self: &State, n: i32) -> Result<(), String> {
        match unsafe { js_pcall(self.state, n) } {
            0 => Ok(()),
            _ => {
                let err = self.tostring(-1);
                assert!(err.is_ok());
                Err(err.ok().unwrap())
            }
        }
    }

    /// Call constructor pushed on stack
    ///
    /// This is similar to State::call(), but without pushing a this
    /// value.
    ///
    /// 1. push the constructure function to call onto the strack
    /// 2. push the arguments to the constructor function in order
    /// 3. finallt, call construct() with the number of arguments
    ///    pushed on the stack
    ///
    /// # Examples
    ///
    /// ```
    /// use mujs;
    ///
    /// let state = mujs::State::new(mujs::JS_STRICT);
    ///
    /// state.dostring("function Car(make, model, year) { \
    ///                                this.make = make; \
    ///                                this.model = model; \
    ///                                this.year = year; \
    ///                              }").unwrap();
    ///
    /// state.getglobal("Car");
    /// state.pushstring("Volvo");
    /// state.pushstring("V50");
    /// state.pushnumber(2010.0);
    /// assert!(state.construct(3).is_ok());
    ///
    /// state.getproperty(0, "model");
    /// println!("Model: {:?}", state.tostring(1).unwrap());
    ///
    /// ```
    pub fn construct(self: &State, n: i32) -> Result<(), String> {
        match unsafe { js_pconstruct(self.state, n) } {
            0 => Ok(()),
            _ => {
                let err = self.tostring(-1);
                assert!(err.is_ok());
                Err(err.ok().unwrap())
            }
        }
    }

    pub fn dostring(self: &State, source: &str) -> Result<(), String> {
        let source_c_str = CString::new(source).unwrap();
        match unsafe {js_dostring(self.state, source_c_str.as_ptr()) } {
            0 => Ok(()),
            _ => {
                let err = self.tostring(-1);
                assert!(err.is_ok());
                Err(err.ok().unwrap())
            }
        }
    }

    /// Throws error on stack
    ///
    /// Pop the error object on the top of the stack and return
    /// control flow to the most recent protected environment.
    ///
    /// # Examples
    ///
    /// ```rust,should_panic
    /// use mujs;
    ///
    /// let state = mujs::State::new(mujs::JS_STRICT);
    ///
    /// state.newerror("Lets create an error");
    /// state.throw();
    /// ```
    pub fn throw(self: &State) {
        unsafe { js_throw(self.state) };
    }

    ///  Push a Error onto the stack
    pub fn newerror(self: &State, message: &str) {
        unsafe { js_newerror(self.state, message.as_ptr() as *const c_char) };
    }

    /// Push an EvaluationError onto the stack
    pub fn newevalerror(self: &State, message: &str) {
        unsafe { js_newevalerror(self.state, message.as_ptr() as *const c_char) }
    }

    /// Push a RangeError onto the stack
    pub fn newrangeerror(self: &State, message: &str) {
        unsafe { js_newrangeerror(self.state, message.as_ptr() as *const c_char) }
    }

    /// Push a ReferenceError onto the stack
    pub fn newreferenceerror(self: &State, message: &str) {
        unsafe { js_newreferenceerror(self.state, message.as_ptr() as *const c_char) }
    }

    /// Push a SyntaxError onto the stack
    pub fn newsyntaxerror(self: &State, message: &str) {
        unsafe { js_newsyntaxerror(self.state, message.as_ptr() as *const c_char) }
    }

    /// Push a TypeError onto the stack
    pub fn newtypeerror(self: &State, message: &str) {
        unsafe { js_newtypeerror(self.state, message.as_ptr() as *const c_char) }
    }

    /// Push a URIError onto the stack
    pub fn newurierror(self: &State, message: &str) {
        unsafe { js_newurierror(self.state, message.as_ptr() as *const c_char) }
    }

    /// Throws an Error in the executing environment
    pub fn error(self: &State, message: &str) {
        unsafe {
            js_newerror(self.state, message.as_ptr() as *const c_char);
            js_throw(self.state);
        };
    }

    /// Throws an EvalError in the executing environment
    pub fn evalerror(self: &State, message: &str) {
        unsafe {
            js_newevalerror(self.state, message.as_ptr() as *const c_char);
            js_throw(self.state);
        }
    }

    /// Throws an RangeError in the executing environment
    pub fn rangeerror(self: &State, message: &str) {
        unsafe {
            js_newrangeerror(self.state, message.as_ptr() as *const c_char);
            js_throw(self.state);
        }
    }

    /// Throws an ReferenceError in the executing environment
    pub fn referenceerror(self: &State, message: &str) {
        unsafe {
            js_newreferenceerror(self.state, message.as_ptr() as *const c_char);
            js_throw(self.state);
        }
    }

    /// Throws an SyntaxError in the executing environment
    pub fn syntaxerror(self: &State, message: &str) {
        unsafe {
            js_newsyntaxerror(self.state, message.as_ptr() as *const c_char);
            js_throw(self.state);
        }
    }

    /// Throws an TypeError in the executing environment
    pub fn typeerror(self: &State, message: &str) {
        unsafe {
            js_newtypeerror(self.state, message.as_ptr() as *const c_char);
            js_throw(self.state);
        }
    }

    /// Throws an URIError in the executing environment
    pub fn urierror(self: &State, message: &str) {
        unsafe {
            js_newurierror(self.state, message.as_ptr() as *const c_char);
            js_throw(self.state);
        }
    }

    /// Get top index of stack
    pub fn gettop(self: &State) -> i32 {
        unsafe {  js_gettop(self.state) }
    }

    /// Pop items off the stack
    ///
    /// # Arguments
    ///
    /// * `n` - Number of items to pop off the stack
    ///
    pub fn pop(self: &State, n: i32) {
        unsafe { js_pop(self.state, n) }
    }

    /// Copy stack item and push on top of stack
    pub fn copy(self: &State, idx: i32) {
        unsafe { js_copy(self.state, idx) }
    }

    /// Create a new object and push onto stack
    pub fn newobject(self: &State) {
        unsafe { js_newobject(self.state) };
    }

    /// Test if stack item is an object
    pub fn isobject(self: &State, idx: i32) -> bool {
        match unsafe { js_isobject(self.state, idx) } {
            0 => false,
            _ => true
        }
    }

    /// Push undefined primitive value onto the stack
    pub fn pushundefined(self: &State) {
        unsafe { js_pushundefined(self.state) };
    }

    /// Push null primitive value onto the stack
    pub fn pushnull(self: &State) {
        unsafe { js_pushnull(self.state) };
    }

    /// Push boolean primitive value onto the stack
    pub fn pushboolean(self: &State, value: bool) {
        match value {
            false => unsafe { js_pushboolean(self.state, 0) },
            true => unsafe { js_pushboolean(self.state, 1) }
        }
    }

    /// Push number primitive value onto the stack
    pub fn pushnumber(self: &State, value: f64) {
        unsafe { js_pushnumber(self.state, value) }
    }

    /// Push string primitive value onto the stack
    pub fn pushstring(self: &State, value: &str) {
        let c_str = CString::new(value).unwrap();
        unsafe { js_pushstring(self.state, c_str.as_ptr()) }
    }

    /// Test if object on stack has named property
    pub fn hasproperty(self: &State, idx: i32, name: &str) -> bool {
        let name_c_str = CString::new(name).unwrap();
        match unsafe { js_hasproperty(self.state, idx, name_c_str.as_ptr()) } {
            0 => false,
            _ => true
        }
    }

    /// Pop the value on top of stack and assigns it to named property
    pub fn setproperty(self: &State, idx: i32, name: &str) {
        let name_c_str = CString::new(name).unwrap();
        unsafe { js_setproperty(self.state, idx, name_c_str.as_ptr()) };
    }

    /// Push the value of named property of object on top of stack
    pub fn getproperty(self: &State, idx: i32, name: &str) {
        let name_c_str = CString::new(name).unwrap();
        unsafe { js_getproperty(self.state, idx, name_c_str.as_ptr()) };
    }

    /// Define named property of object
    ///
    /// # Examples
    ///
    /// ```
    /// use mujs;
    ///
    /// let state = mujs::State::new(mujs::JS_STRICT);
    ///
    /// state.newobject();
    /// state.pushstring("A value");
    /// state.defproperty(0, "value", mujs::JS_DONTCONF);
    ///
    /// ```
    pub fn defproperty(self: &State, idx: i32, name: &str, attrs: PropertyAttributes) {
        let name_c_str = CString::new(name).unwrap();
        unsafe { js_defproperty(self.state, idx, name_c_str.as_ptr(), attrs.bits) };
    }

    /// Define a getter and setter attribute og a property of object on stack
    ///
    /// Pop the two getter and setter functions from the stack. Use
    /// null instead of a function object if you want to leave any of
    /// the functions unset.
    pub fn defaccessor(self: &State, idx: i32, name: &str, attrs: PropertyAttributes) {
        let name_c_str = CString::new(name).unwrap();
        unsafe { js_defaccessor(self.state, idx, name_c_str.as_ptr(), attrs.bits) };
    }

    /// Delete named property of object
    pub fn delproperty(self: &State, idx: i32, name: &str) {
        let name_c_str = CString::new(name).unwrap();
        unsafe { js_delproperty(self.state, idx, name_c_str.as_ptr()) };
    }

    /// Push object representing the global environment record
    pub fn pushglobal(self: &State) {
        unsafe { js_pushglobal(self.state) }
    }

    /// Get named global variable
    pub fn getglobal(self: &State, name: &str) {
        let name_c_str = CString::new(name).unwrap();
        unsafe { js_getglobal(self.state, name_c_str.as_ptr()) }
    }

    /// Set named variable with object on top of stack
    pub fn setglobal(self: &State, name: &str) {
        let name_c_str = CString::new(name).unwrap();
        unsafe { js_setglobal(self.state, name_c_str.as_ptr()) }
    }

    /// Define named global variable
    pub fn defglobal(self: &State, name: &str, attrs: PropertyAttributes) {
        let name_c_str = CString::new(name).unwrap();
        unsafe { js_defglobal(self.state, name_c_str.as_ptr(), attrs.bits) }
    }

    /// Test if item on stack is defined
    pub fn isdefined(self: &State, idx: i32) -> bool {
        match unsafe { js_isdefined(self.state, idx) } {
            0 => false,
            _ => true
        }
    }

    /// Test if item on stack is undefined
    pub fn isundefined(self: &State, idx: i32) -> bool {
        match unsafe { js_isundefined(self.state, idx) } {
            0 => false,
            _ => true
        }
    }

    /// Convert value on stack to string
    pub fn tostring(self: &State, idx: i32) -> Result<String, String> {
        let c_buf: *const c_char = unsafe { js_tostring(self.state, idx) };

        if c_buf == std::ptr::null() {
            return Err("Null string".to_string())
        }

        Ok(unsafe {
            CStr::from_ptr(c_buf).to_string_lossy().into_owned()
        })
    }

    /// Convert value on stack to boolean
    pub fn toboolean(self: &State, idx: i32) -> Result<bool, String> {
        match unsafe { js_toboolean(self.state, idx) } {
            0 => Ok(false),
            _ => Ok(true)
        }
    }

    /// Convert value on stack to number
    pub fn tonumber(self: &State, idx: i32) -> Result<f64, String> {
        Ok( unsafe { js_tonumber(self.state, idx) } )
    }

}

impl Drop for State {
    fn drop(self: &mut State) {
        unsafe { js_freestate(self.state) };
    }
}


#[cfg(test)]
mod tests {
    use std;
    #[test]
    fn create_new_state() {
        let _ = ::State::new(::StateFlags{bits: 0});
    }

    #[test]
    #[should_panic]
    fn panic_are_handled() {
        let state = ::State::new(::StateFlags{bits: 0});
        unsafe {
            ::js_pushnumber(state.state, 1.234);
            ::js_pushnull(state.state);
            ::js_call(state.state, 0);
        }
    }

    #[test]
    fn call_garbage_collector() {
        let state = ::State::new(::StateFlags{bits: 0});
        state.gc(false);
    }

    #[test]
    fn loadstring_with_broken_script() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.loadstring("myscript", "func broken() {").is_err());
    }

    #[test]
    fn loadstring_with_complete_script() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.loadstring("myscript", "func broken() { return Math.sin(3.2); };").is_err());
    }

    #[test]
    fn call_with_runtime_error() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.loadstring("myscript", "mystic.func();").is_ok());
        state.newobject();
        assert!(state.call(0).is_err());
    }

    #[test]
    fn call_with_success() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.loadstring("myscript", "Math.sin(3.2);").is_ok());
        state.newobject();
        assert!(state.call(0).is_ok());
    }

    #[test]
    fn construct_with_success() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.dostring("function func(a) { this.a = a; }").is_ok());
        state.getglobal("func");
        state.pushnumber(1.1234);
        assert!(state.construct(1).is_ok());
        state.getproperty(0,"a");
        assert_eq!(state.tonumber(1).unwrap(), 1.1234);
    }

    #[test]
    fn dostring_with_success() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.dostring("Math.sin(3.2);").is_ok());
    }

    #[test]
    fn dostring_with_broken_script() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.dostring("func broken() {").is_err());
    }

    #[test]
    fn dostring_with_runtime_error() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.dostring("mystic.func();").is_err());
    }

    #[test]
    fn tostring_ascii() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.loadstring("myscript", "'Hello' + ' ' + 'World!';").is_ok());
        state.newobject();
        assert!(state.call(0).is_ok());
        assert_eq!(state.tostring(0).ok().unwrap(), "Hello World!");
    }

    #[test]
    fn tostring_utf8() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.loadstring("myscript", "'Hello' + ' ' + 'Båsse!';").is_ok());
        state.newobject();
        assert!(state.call(0).is_ok());
        assert_eq!(state.tostring(0).ok().unwrap(), "Hello Båsse!");
    }

    #[test]
    fn tostring_with_number_on_stack() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.loadstring("myscript", "240.32;").is_ok());
        state.newobject();
        assert!(state.call(0).is_ok());
        assert_eq!(state.tostring(0).ok().unwrap(), "240.32");
    }

    #[test]
    fn toboolean_with_true_on_stack() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.loadstring("myscript", "true").is_ok());
        state.newobject();
        assert!(state.call(0).is_ok());
        assert_eq!(state.toboolean(0).ok().unwrap(), true);
    }

    #[test]
    fn toboolean_with_false_on_stack() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.loadstring("myscript", "false").is_ok());
        state.newobject();
        assert!(state.call(0).is_ok());
        assert_eq!(state.toboolean(0).ok().unwrap(), false);
    }

    #[test]
    fn toboolean_with_positive_number_on_stack() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.loadstring("myscript", "1").is_ok());
        state.newobject();
        assert!(state.call(0).is_ok());
        assert_eq!(state.toboolean(0).ok().unwrap(), true);
    }

    #[test]
    fn toboolean_with_zero_number_on_stack() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.loadstring("myscript", "0").is_ok());
        state.newobject();
        assert!(state.call(0).is_ok());
        assert_eq!(state.toboolean(0).ok().unwrap(), false);
    }

    #[test]
    fn toboolean_with_null_on_stack() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.loadstring("myscript", "null").is_ok());
        state.newobject();
        assert!(state.call(0).is_ok());
        assert_eq!(state.toboolean(0).ok().unwrap(), false);
    }

    #[test]
    fn toboolean_with_undefined_on_stack() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.loadstring("myscript", "undefined").is_ok());
        state.newobject();
        assert!(state.call(0).is_ok());
        assert_eq!(state.toboolean(0).ok().unwrap(), false);
    }

    #[test]
    fn tonumber_with_positive_number_on_stack() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.loadstring("myscript", "1.53278").is_ok());
        state.newobject();
        assert!(state.call(0).is_ok());
        assert_eq!(state.tonumber(0).ok().unwrap(), 1.53278);
    }

    #[test]
    fn tonumber_with_negative_number_on_stack() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.loadstring("myscript", "-1.53278;").is_ok());
        state.newobject();
        assert!(state.call(0).is_ok());
        assert_eq!(state.tonumber(0).ok().unwrap(), -1.53278);
    }

    #[test]
    fn tonumber_with_valid_string_on_stack() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.loadstring("myscript", "'1.53278'").is_ok());
        state.newobject();
        assert!(state.call(0).is_ok());
        assert_eq!(state.tonumber(0).ok().unwrap(), 1.53278);
    }

    #[test]
    fn tonumber_with_invalid_string_on_stack() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.loadstring("myscript", "'hello world'").is_ok());
        state.newobject();
        assert!(state.call(0).is_ok());
        assert_eq!(state.tonumber(0).ok().unwrap().classify(), std::num::FpCategory::Nan);
    }

    #[test]
    fn pushundefined_verify_as_string() {
        let state = ::State::new(::StateFlags{bits: 0});
        state.pushundefined();
        assert_eq!(state.tostring(0).ok().unwrap(), "undefined");
     }

    #[test]
    fn pushnull_verify_as_string() {
        let state = ::State::new(::StateFlags{bits: 0});
        state.pushnull();
        assert_eq!(state.tostring(0).ok().unwrap(), "null");
    }

    #[test]
    fn pushboolean_verify_as_string() {
        let state = ::State::new(::StateFlags{bits: 0});
        state.pushboolean(true);
        assert_eq!(state.tostring(0).ok().unwrap(), "true");
    }

    #[test]
    fn pushnumber_verify_as_string() {
        let state = ::State::new(::StateFlags{bits: 0});
        state.pushnumber(1.234);
        assert_eq!(state.tostring(0).ok().unwrap(), "1.234");
    }

    #[test]
    fn pushstring_ascii() {
        let state = :: State::new(::StateFlags{bits: 0});
        state.pushstring("Hello World!");
        assert_eq!(state.tostring(0).ok().unwrap(), "Hello World!");
    }

    #[test]
    fn pushstring_utf8() {
        let state = :: State::new(::StateFlags{bits: 0});
        state.pushstring("Hello Båsse!");
        assert_eq!(state.tostring(0).ok().unwrap(), "Hello Båsse!");
    }

    #[test]
    fn newerror_verify_as_string() {
        let state = ::State::new(::StateFlags{bits: 0});
        state.newerror("This is an error");
        assert_eq!(state.tostring(0).ok().unwrap(), "Error: This is an error");
    }

    #[test]
    fn newevalerror_verify_as_string() {
        let state = ::State::new(::StateFlags{bits: 0});
        state.newevalerror("This is an error");
        assert_eq!(state.tostring(0).ok().unwrap(), "EvalError: This is an error");
    }

    #[test]
    fn newrangeerror_verify_as_string() {
        let state = ::State::new(::StateFlags{bits: 0});
        state.newrangeerror("This is an error");
        assert_eq!(state.tostring(0).ok().unwrap(), "RangeError: This is an error");
    }

    #[test]
    fn newreferenceerror_verify_as_string() {
        let state = ::State::new(::StateFlags{bits: 0});
        state.newreferenceerror("This is an error");
        assert_eq!(state.tostring(0).ok().unwrap(), "ReferenceError: This is an error");
    }

    #[test]
    fn newsyntaxerror_verify_as_string() {
        let state = ::State::new(::StateFlags{bits: 0});
        state.newsyntaxerror("This is an error");
        assert_eq!(state.tostring(0).ok().unwrap(), "SyntaxError: This is an error");
    }

    #[test]
    fn newtypeerror_verify_as_string() {
        let state = ::State::new(::StateFlags{bits: 0});
        state.newtypeerror("This is an error");
        assert_eq!(state.tostring(0).ok().unwrap(), "TypeError: This is an error");
    }

    #[test]
    fn newurierror_verify_as_string() {
        let state = ::State::new(::StateFlags{bits: 0});
        state.newurierror("This is an error");
        assert_eq!(state.tostring(0).ok().unwrap(), "URIError: This is an error");
    }

    #[test]
    #[should_panic(expected = "Error: This is an error")]
    fn error_should_panic() {
        let state = ::State::new(::StateFlags{bits: 0});
        state.error("This is an error");
    }

    #[test]
    #[should_panic(expected = "EvalError: This is an error")]
    fn evalerror_should_panic() {
        let state = ::State::new(::StateFlags{bits: 0});
        state.evalerror("This is an error");
    }

    #[test]
    #[should_panic(expected = "RangeError: This is an error")]
    fn rangeerror_should_panic() {
        let state = ::State::new(::StateFlags{bits: 0});
        state.rangeerror("This is an error");
    }

    #[test]
    #[should_panic(expected = "ReferenceError: This is an error")]
    fn referenceerror_should_panic() {
        let state = ::State::new(::StateFlags{bits: 0});
        state.referenceerror("This is an error");
    }

    #[test]
    #[should_panic(expected = "SyntaxError: This is an error")]
    fn syntaxerror_should_panic() {
        let state = ::State::new(::StateFlags{bits: 0});
        state.syntaxerror("This is an error");
    }

    #[test]
    #[should_panic(expected = "TypeError: This is an error")]
    fn typeerror_should_panic() {
        let state = ::State::new(::StateFlags{bits: 0});
        state.typeerror("This is an error");
    }

    #[test]
    #[should_panic(expected = "URIError: This is an error")]
    fn urierror_should_panic() {
        let state = ::State::new(::StateFlags{bits: 0});
        state.urierror("This is an error");
    }

    #[test]
    fn gettop_with_empty_stack() {
        let state = ::State::new(::JS_STRICT);
        assert_eq!(state.gettop(), 0);
    }

    #[test]
    fn gettop_with_one_item_on_stack() {
        let state = ::State::new(::JS_STRICT);
        state.pushundefined();
        assert_eq!(state.gettop(), 1);
    }

    #[test]
    fn gettop_with_five_items_on_stack() {
        let state = ::State::new(::JS_STRICT);
        state.pushnumber(1.0);
        state.pushnumber(2.0);
        state.pushnumber(3.0);
        state.pushnumber(4.0);
        state.pushnumber(5.0);
        assert_eq!(state.gettop(), 5);
    }

    #[test]
    fn pop_with_empty_stack() {
        let state = ::State::new(::JS_STRICT);
        state.pop(0);
        assert_eq!(state.gettop(), 0);
    }

    #[test]
    fn pop_one_and_only_item_on_stack() {
        let state = ::State::new(::JS_STRICT);
        state.pushnumber(1.0);
        state.pop(1);
        assert_eq!(state.gettop(), 0);
    }

    #[test]
    fn pop_one_of_two_items_on_stack() {
        let state = ::State::new(::JS_STRICT);
        state.pushnumber(1.0);
        state.pushnumber(2.0);
        state.pop(1);
        assert_eq!(state.gettop(), 1);
    }

    #[test]
    fn copy_one_item_on_stack() {
        let state = ::State::new(::JS_STRICT);
        state.pushnumber(1.2345);
        state.copy(0);
        assert_eq!(state.tonumber(0).unwrap(), 1.2345);
        assert_eq!(state.tonumber(1).unwrap(), 1.2345);
    }

    #[test]
    fn copy_of_item_copies_on_stack() {
        let state = ::State::new(::JS_STRICT);
        state.pushnumber(1.2345);
        state.copy(0);
        state.copy(1);
        state.copy(2);
        assert_eq!(state.tonumber(0).unwrap(), 1.2345);
        assert_eq!(state.tonumber(1).unwrap(), 1.2345);
        assert_eq!(state.tonumber(2).unwrap(), 1.2345);
        assert_eq!(state.tonumber(3).unwrap(), 1.2345);
    }

    #[test]
    fn isdefined_on_undefined_is_false() {
        let state = ::State::new(::StateFlags{bits: 0});
        state.pushundefined();
        assert_eq!(state.isdefined(0), false);
    }

    #[test]
    fn isdefined_on_number_is_true() {
        let state = ::State::new(::StateFlags{bits: 0});
        state.pushnumber(1.234);
        assert_eq!(state.isdefined(0), true);
    }

    #[test]
    fn isundefined_on_undefined_is_true() {
        let state = ::State::new(::StateFlags{bits: 0});
        state.pushundefined();
        assert_eq!(state.isundefined(0), true);
    }

    #[test]
    fn isundefined_on_number_is_false() {
        let state = ::State::new(::StateFlags{bits: 0});
        state.pushnumber(1.234);
        assert_eq!(state.isundefined(0), false);
    }

    #[test]
    fn isobject_on_object_is_true() {
        let state = ::State::new(::StateFlags{bits: 0});
        state.newobject();
        assert_eq!(state.isobject(0), true);
    }

    #[test]
    fn isobject_on_number_is_false() {
        let state = ::State::new(::StateFlags{bits: 0});
        state.pushnumber(1.234);
        assert_eq!(state.isobject(0), false);
    }

    #[test]
    fn hasproperty_on_object_with_existing_property() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.loadstring("myscript", "var person = {name: \"Tester\", age: 32}; person").is_ok());
        state.newobject();
        assert!(state.call(0).is_ok());
        assert_eq!(state.hasproperty(0, "age"), true);
    }

    #[test]
    fn hasproperty_on_object_with_non_existing_property() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.loadstring("myscript", "var person = {name: \"Tester\", age: 32}; person").is_ok());
        state.newobject();
        assert!(state.call(0).is_ok());
        assert_eq!(state.hasproperty(0, "phone"), false);
    }

    #[test]
    fn getproperty_on_object_with_existing_property() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.loadstring("myscript", "var person = {name: \"Tester\", age: 32}; person").is_ok());
        state.newobject();
        assert!(state.call(0).is_ok());
        state.getproperty(0, "name");
        assert_eq!(state.tostring(1).ok().unwrap(), "Tester");
    }

    #[test]
    fn getproperty_on_object_with_non_existing_property() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.loadstring("myscript", "var person = {name: \"Tester\", age: 32}; person").is_ok());
        state.newobject();
        assert!(state.call(0).is_ok());
        state.getproperty(0, "phone");
        assert_eq!(state.isundefined(1), true);
    }

    #[test]
    fn setproperty_on_object_as_number_value() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.loadstring("myscript", "var person = {name: \"Tester\", age: 32}; person").is_ok());
        state.newobject();
        assert!(state.call(0).is_ok());
        state.pushnumber(1.234);
        state.setproperty(0, "age");
        state.getproperty(0, "age");
        assert_eq!(state.tonumber(1).unwrap(), 1.234);
    }

    #[test]
    fn setproperty_on_object_changing_to_number_value() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.loadstring("myscript", "var person = {name: \"Tester\", age: 32}; person").is_ok());
        state.newobject();
        assert!(state.call(0).is_ok());
        state.pushnumber(1.234);
        state.setproperty(0, "name");
        state.getproperty(0, "name");
        assert_eq!(state.tonumber(1).unwrap(), 1.234);
    }


    #[test]
    fn setproperty_on_object_non_existing_property() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.loadstring("myscript", "var person = {name: \"Tester\", age: 32}; person").is_ok());
        state.newobject();
        assert!(state.call(0).is_ok());
        state.pushnumber(1.234);
        state.setproperty(0, "phone");
        state.getproperty(0, "phone");
        assert_eq!(state.tonumber(1).unwrap(), 1.234);
    }

    #[test]
    fn defproperty_read_only_on_object() {
        let state = ::State::new(::StateFlags{bits: 0});
        state.newobject();
        state.pushnumber(1.234);
        state.defproperty(0, "age", ::JS_READONLY);

        state.pushnumber(1.0);
        state.setproperty(0, "age");
        state.getproperty(0, "age");
        assert_eq!(state.tonumber(1).unwrap(), 1.234);
    }

    #[test]
    fn defproperty_on_object() {
        let state = ::State::new(::StateFlags{bits: 0});
        state.newobject();
        state.pushnumber(1.234);
        state.defproperty(0, "age", ::PropertyAttributes{bits: 0});

        state.pushnumber(1.0);
        state.setproperty(0, "age");
        state.getproperty(0, "age");
        assert_eq!(state.tonumber(1).unwrap(), 1.0);
    }

    #[test]
    fn delproperty_on_not_configurable_property() {
        let state = ::State::new(::StateFlags{bits: 0});
        let source = "var obj = {}; \
                      Object.defineProperty(obj, 'func', { \
                        value: function (a, b) { return a + b; },\
                      }); obj";

        assert!(state.loadstring("myscript", source).is_ok());
        state.newobject();
        assert!(state.call(0).is_ok());

        state.delproperty(0, "func");
        state.getproperty(0, "func");
        assert_eq!(state.tostring(1).unwrap(), "function (a,b) { ... }");
    }

    #[test]
    fn delproperty_on_configurable_property() {
        let state = ::State::new(::StateFlags{bits: 0});
        let source = "var obj = {}; \
                      Object.defineProperty(obj, 'func', { \
                        value: function (a, b) { return a + b; },\
                        configurable: true
                      }); obj";

        assert!(state.loadstring("myscript", source).is_ok());
        state.newobject();
        assert!(state.call(0).is_ok());

        state.delproperty(0, "func");
        state.getproperty(0, "func");
        assert_eq!(state.tostring(1).unwrap(), "undefined");
    }

    #[test]
    fn setglobal_on_state() {
        let state = ::State::new(::StateFlags{bits: 0});
        state.newobject();
        state.pushnumber(1.234);
        state.setproperty(-2, "age");
        state.setglobal("me");

        assert!(state.loadstring("myscript", "me").is_ok());
        state.newobject();
        assert!(state.call(0).is_ok());
        state.getproperty(0, "age");
        assert_eq!(state.tostring(1).unwrap(), "1.234");
    }

    #[test]
    fn defglobal_on_state_readonly() {
        let attrs = ::JS_READONLY;
        let state = ::State::new(::StateFlags{bits: 0});
        state.newobject();
        state.pushnumber(1.234);
        state.setproperty(-2, "age");
        state.defglobal("me", attrs);

        state.newobject();
        state.pushnumber(1.0);
        state.setproperty(-2, "age");
        state.setglobal("me");

        assert!(state.loadstring("myscript", "me").is_ok());
        state.newobject();
        assert!(state.call(0).is_ok());

        state.getproperty(0, "age");
        assert_eq!(state.tonumber(1).unwrap(), 1.234);
    }

    #[test]
    fn defglobal_on_state_is_writeable() {
        let attrs = ::PropertyAttributes{bits: 0};
        let state = ::State::new(::StateFlags{bits: 0});
        state.newobject();
        state.pushnumber(1.234);
        state.setproperty(-2, "age");
        state.defglobal("me", attrs);

        state.newobject();
        state.pushnumber(1.0);
        state.setproperty(-2, "age");
        state.setglobal("me");

        assert!(state.loadstring("myscript", "me").is_ok());
        state.newobject();
        assert!(state.call(0).is_ok());

        state.getproperty(0, "age");
        assert_eq!(state.tonumber(1).unwrap(), 1.0);
    }

    #[test]
    fn getglobal_from_script() {
        let state = ::State::new(::StateFlags{bits: 0});
        assert!(state.loadstring("myscript", "var me = {age: 1.234};").is_ok());
        state.newobject();
        assert!(state.call(0).is_ok());

        state.getglobal("me");
        state.getproperty(1, "age");
        assert_eq!(state.tostring(2).unwrap(), "1.234");
    }
}
