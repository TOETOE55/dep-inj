# Exploring Design Patterns in Rust: Inter-Component Interface Invocation

In software design, it is common to break down a complex program into several components. These components are then assembled and managed through a container, and the program exposes its functionality to the outside world through this container.

In the ideal scenario, these components should be independent of each  other, meaning they do not directly call or access other parts of the  container. They are like bricks stacked together to form a program. Such a design is certainly desirable but can be challenging to achieve. In a complex project, there are inevitably situations where modules need to  interact with each other.

To fulfill this requirement, there are several approaches, and among them, Method 5 is the best one I came up with:



# Method 1: Holding a Weak Reference to the Container within Components

In this approach, each component holds a reference (or weak reference) to the container, allowing access to other parts of the container.

However, it is worth noting that this design approach is less commonly used in Rust. Rust lacks built-in support for writing mutually dependent structures, making it cumbersome and prone to bugs when implementing such a design.

```rust
struct ComponentA {
    container: Weak<Container>,
    // ...
}

struct Container {
    component_a: Arc<ComponentA>,
    component_b: Arc<ComponentB>,
    component_c: Arc<ComponentC>,
    // ...
}
```



# Method 2: Injecting the Container's Reference through Parameters

In this approach, all functions in the program pass the reference to the entire container as a parameter. Consequently, a component can access the methods provided by other components through this reference.

However, this approach also has some issues. It requires the component and the container to be defined in the same crate. If the container is defined in a crate higher than the component, the component won't have access to the container's type, and it won't be able to define a method that requires a container reference.

```rust
struct Container {
    component_a: ComponentA,
    component_b: ComponentB,
    component_c: ComponentC,
    // ...
}

impl ComponentA {
    fn foo(container: &Container) {
        // Accessing methods provided by another component can be done through the Container.
        container.component_b.bar(
            // Components within the same module can access each other's fields directly.
            container.component_a.x
        );
    }
}
```



# Method 3: Interface Segregation - Centralized Interface Layer

As the project becomes more complex, it is common to organize components using crates rather than mod. The dependency graph of crates would look like the following:

[![](https://mermaid.ink/img/pako:eNqFT7sOwjAM_JXKc_sDGZCgXZlgQlmsxKWRGrsyjhCq-u8EGBh7072k060QJBI4GGd5hgnVmuvg1XNT0QsbJiZtuu5QVV6Eie24k5928h5ayKQZU6zD66frwSbK5MFVGmnEMpsHz1utYjG5vDiAMy3UQlkiGg0J74oZ3Ijzo7oL8k3krykmEz3_zn0_bm8z0U8u?type=png)](https://mermaid.live/edit#pako:eNqFT7sOwjAM_JXKc_sDGZCgXZlgQlmsxKWRGrsyjhCq-u8EGBh7072k060QJBI4GGd5hgnVmuvg1XNT0QsbJiZtuu5QVV6Eie24k5928h5ayKQZU6zD66frwSbK5MFVGmnEMpsHz1utYjG5vDiAMy3UQlkiGg0J74oZ3Ijzo7oL8k3krykmEz3_zn0_bm8z0U8u)

However, one limitation of using crates is that crates cannot have  direct mutual dependencies, which means that the interfaces provided by  one crate cannot be directly invoked by another crate (although there is a workaround through a crate called [linkme](https://crates.io/crates/linkme) that enables inter-crate invocations during the linking phase).

To address this issue, one possible solution is to introduce a middle layer that houses all the **state** of the program and the **interfaces** provided by the components. Meanwhile, the **implementation** and **registration** of the interfaces can be placed in the component layer. This structure can be visualized as shown in the following diagram:

[![](https://mermaid.ink/img/pako:eNqFkEELgzAMhf-K5Kx_oIfB1Kun7TR6CTbOQptKlzKG-N_XOYaDwcwpL98j8N4MfTAECgYX7v2IUYpzq6PmIk8TWNAyxaKqDln5KTCxHHd4vcObjX8-robOGuPoB9b_YPMFoQRP0aM1Oc_8MmqQkTxpUHk1NGByokHzkq2YJJwe3IOSmKiENBkUai1eI3pQA7pbvk7IlxA2TcZKiN27s7W65Qn1EWrK?type=png)](https://mermaid.live/edit#pako:eNqFkEELgzAMhf-K5Kx_oIfB1Kun7TR6CTbOQptKlzKG-N_XOYaDwcwpL98j8N4MfTAECgYX7v2IUYpzq6PmIk8TWNAyxaKqDln5KTCxHHd4vcObjX8-robOGuPoB9b_YPMFoQRP0aM1Oc_8MmqQkTxpUHk1NGByokHzkq2YJJwe3IOSmKiENBkUai1eI3pQA7pbvk7IlxA2TcZKiN27s7W65Qn1EWrK)

 `rustc`follows a similar approach. It defines the `GlobalCtxt<'tcx>` struct within the middle layer, which encompasses all the compiler's state. Additionally, it defines the `Providers` struct, which contains all the interfaces exposed by the compiler.  These interfaces typically include a parameter that references the `GlobalCtxt`, allowing components to implement and register these methods within  different compiler modules. Consequently, the components can also invoke these methods as needed.

a centralized middle layer has its limitations and challenges:

1. The middle layer tends to become a large codebase, and all components depend on it. Whenever there is a need for recompilation (e.g., modifying code in the middle layer), the entire project often needs to be recompiled, which can result in significantly longer build times. Parallel compilation is also not fully utilized, further exacerbating the compile-time explosion.
2. The interface declarations in the middle layer have some restrictions, and they cannot include generic functions. This limitation can restrict the flexibility and expressiveness of the interfaces exposed by the middle layer.
3. Components cannot have private states that are not exposed in the middle layer. All state related to components must be exposed in the middle layer, potentially leading to a loss of encapsulation and making it harder to manage private component-specific state.
4. The component interfaces are dynamically provided, meaning that at compile time, it is not known whether the called interfaces have been implemented or not. This dynamic nature can introduce additional complexity and potential runtime errors.
5. The middle layer is typically a single crate, making it challenging to separate and utilize only a subset of components when needed. This can hinder the ability to compose and reuse components independently, although it may be a relatively niche requirement.



# Method 4: Interface Segregation - Dynamic Querying

To address some of the issues in Method 3, we can further split the Component layer into an Implementation layer and an Interface layer. The interface declarations from the Middle layer can be moved to the Component Interface layer, while the state associated with the Middle layer and components can be placed in the Component Implementation layer. The Middle layer then retains interfaces for interface registration, interface lookup, and state registration.

This revised structure can be visualized as follows:

[![](https://mermaid.ink/img/pako:eNqN0r0KwjAQB_BXKTfrC3QQ2nRxKAg6SZajuWogHyVeEZG-u1GHVgtpM-VyP7gL_J_QeEWQQ2v8vbli4OxUySBdFo_wjlE7Ctl2u4uV7bwjx8XedmbBlCuMmJrRTqf8DS4O-zWunLiZL9M-5YqVTqTmi7RPueS_3st9eK2VMjTfc6EvfvuwAUvBolYxHM-3lcBXsiQhj1dFLfaGJUg3RIo9--PDNZBz6GkDfaeQqdJ4CWghb9Hc4muH7uz9WJPS7EP9DeAnh8MLewPXWA?type=png)](https://mermaid.live/edit#pako:eNqN0r0KwjAQB_BXKTfrC3QQ2nRxKAg6SZajuWogHyVeEZG-u1GHVgtpM-VyP7gL_J_QeEWQQ2v8vbli4OxUySBdFo_wjlE7Ctl2u4uV7bwjx8XedmbBlCuMmJrRTqf8DS4O-zWunLiZL9M-5YqVTqTmi7RPueS_3st9eK2VMjTfc6EvfvuwAUvBolYxHM-3lcBXsiQhj1dFLfaGJUg3RIo9--PDNZBz6GkDfaeQqdJ4CWghb9Hc4muH7uz9WJPS7EP9DeAnh8MLewPXWA)

For example, the Middle layer provides a type that is used to store the states of all components and the interfaces they provide:

```rust
pub struct GlobalCtxt {
    /// All the components will be registered in here.
    components: HashMap<TypeId, Rc<dyn Any>>,
    /// All the interfaces provided by the components will registered in here.
    interfaces: HashMap<TypeId, Rc<dyn Any>>,
}

impl GlobalCtxt {
    pub fn register_component<Component: Any>(
        &mut self, 
        component: Rc<Component>
    ) -> Result<()> { /*...*/ }
    
    pub fn register_interface<Interface: Any>(
        &mut self, 
        interface: Rc<Interface>
    ) -> Result<()> { /*...*/ }
    
    pub fn get_interface<Interface: Any + ?Sized>(&self) 
        -> Option<Rc<Interface>>
    { /*...*/ }
}
```

The Component Interface layer, on the other hand, provides a set of interface definitions：

```rust
// ComponentFooAPI
trait Foo {
    fn foo(&self, ctxt: &GlobalCtxt);
}
```

The Component Implementation layer then implements the interfaces declared in the Interface layer and registers them with the `GlobalCtxt`.

```rust
struct ComponentFoo {
    count: RefCell<usize>,
}

impl Foo for ComponentFoo {
    fn foo(&self, ctxt: &GlobalCtxt) {
        *self.count.borrow_mut() += 1;
        // Invoke the interfaces provided by other components through `GlobalCtxt`.
        let bar = ctxt.get_interface::<dyn Bar>().unwrap();
        bar.bar();
    }
}

pub fn register(ctxt: &mut ClobalCtxt) -> Result<()> {
    let component = Rc::new(ComponentFoo::default());
    register.register_component(component.clone())?;
    register.register_interface(component as Rc<dyn Foo>)
}
```

This approach addresses some of the issues in Method 3:

1. Slow compilation: By separating the responsibilities and reducing the codebase in the Middle layer, it no longer requires extensive compilation. Additionally, placing interface definitions in multiple crates allows for parallel compilation, optimizing the compilation speed.
2. Inability to split components: With this solution, the Middle layer doesn't include the code of all components. Each component has its own separate crates, making it easier to split them as needed.
3. Inability to encapsulate component states: In this approach, the states of components are defined within their respective implementation layers. The Middle layer's type is erased, ensuring that component states are not leaked or exposed.

But there are some additional challenges and trade-offs introduced with this approach:

1. Limited interface form: The example provided here demonstrates interfaces using traits that satisfy the `Any` trait and are dyn safe. This implies that the interfaces have certain restrictions and may not support more complex or generic interface forms.
2. Dynamic interface provisioning: Similarly, this approach doesn't allow us to determine at compile time whether a particular interface has been provided or implemented. It relies on dynamic checks during runtime to determine if the interface is available.
3. Overhead of interface querying: The introduction of interface querying using a `HashMap` with `TypeId` incurs additional overhead compared to Method 3. The lookup process adds extra computational cost and may impact performance compared to direct method calls or static dispatch.



# Method 5: Interface Implementation Separation - Static Dependency Injection

Is there a solution that can simultaneously address these issues:

1. Support inter-crate communication.
2. No or minimal restrictions on interface forms.
3. Zero-cost interface calls.
4. Static provision of interfaces.
5. Non-exposure of component states.
6. Component modularity.
7. Fast compilation speed.

Recently, I may have found a potential solution that could fulfill these requirements after coming across a library called [`ref-cast`](https://docs.rs/ref-cast/1.0.16/ref_cast/).

> The `ref-cast` library provides macros for generating `&T -> &Wrapper<T>` conversion methods.

This approach also divides the components into implementation and  interface layers, but no longer requires a middle layer. Instead, all  integration operations are placed in the `Container` layer:

[![](https://mermaid.ink/img/pako:eNqN0r0KgzAQB_BXkZv1BRwKGheHQqGdSpbDnFXIh6QnpYjv3tgOSgupme6OX_gHchM0ThHk0Gr3aDr0nFwq6aVNwhHOMvaWfJJlh9CZwVmyXNRm0H9MucOIrVntNuUruDjVe1y5cT--jPuYK3Y6EcsXcR9zyzshBUPeYK_Cp03LHQnckSEJeSgVtThqliDtHCiO7M5P20DOfqQUxkEhU9XjzaOBvEV9D9MB7dW5tSfVs_PHz2K892N-AfLdtxY?type=png)](https://mermaid.live/edit#pako:eNqN0r0KgzAQB_BXkZv1BRwKGheHQqGdSpbDnFXIh6QnpYjv3tgOSgupme6OX_gHchM0ThHk0Gr3aDr0nFwq6aVNwhHOMvaWfJJlh9CZwVmyXNRm0H9MucOIrVntNuUruDjVe1y5cT--jPuYK3Y6EcsXcR9zyzshBUPeYK_Cp03LHQnckSEJeSgVtThqliDtHCiO7M5P20DOfqQUxkEhU9XjzaOBvEV9D9MB7dW5tSfVs_PHz2K892N-AfLdtxY)

Let's take a closer look at how this method is organized:

1. **Interface Layer**. Let's assume we have two components, `Even` and `Odd`, each with its own set of interfaces:

   ```rust
   // even-api
   trait Even {
       fn is_even(&mut self, n: u64) -> bool;
       /// Emit the number of times `Even::is_even` is called.
       // This method makes `Even` no longer dyn safe.
       fn emit_count<T>(&self, f: impl FnOnce(usize) -> T) -> T;
   }
   
   // odd-api
   trait Odd {
       fn is_odd(&mut self, n: u64) -> bool;
   }
   ```

2. **Implementation Layer**. In this implementation layer, we introduce a `Proxy` structure that takes a generic parameter for injecting interfaces and  the component's own state. All the functionality related to the  component can be implemented based on the `Proxy` structure:

   ```rust
   // even-impl
   
   #[derive(Defaule)]
   pub struct EvenState {
       count: usize,
   }
   
   #[derive(ref_cast::RefCast)]
   #[repr(transparent)]
   pub struct EvenProxy<Ctx: ?Sized> {
       ctx: Ctx
   };
   
   // It allows `&EvenProxy<Ctx>` to be used as `&EvenState`.
   impl<Ctx: ?Sized> Deref for EvenProxy<Ctx> 
   where
       Ctx: AsRef<EvenState>,
   {
       type Target = EvenState;
       fn deref(&self) -> &EvenState {
           self.ctx.as_ref()
       }
   }
   
   // It allows `&mut EvenProxy<Ctx>` to be used as `&mut EvenState`.
   impl<Ctx: ?Sized> Deref for EvenProxy<Ctx> 
   where
       Ctx: AsRef<EvenState>,
       Ctx: AsMut<EvenState>
   {
       fn deref_mut(&mut self) -> &mut EvenState {
           self.ctx.as_mut()
       }
   }
   
   
   // implement Even interface for Even Component
   impl<Ctx: ?Sized> Even for EvenProxy<Ctx>
   where
       // Injecting `Odd` through `Ctx`.
       Ctx: Odd,
   {
       fn is_even(&mut self, n: u64) -> bool {
          is_even_impl(self, n)
       }
       
       fn emit_count<T>(&self, f: impl FnOnce(usize) -> T) -> T {
           f(self.count)
       }
   }
   
   fn is_even_impl(
       // Using trait object theoretically can speed up compilation because the code generation doesn't need to wait for generic instantiation.
       proxy: &mut EvenProxy<dyn Odd>,
       n: u64,
   ) -> bool {
       proxy.count += 1;
   
       (n == 0) || proxy
           .ctx // invoke is_odd through ctx
           .is_odd(n - 1)
   }
   ```

   The implementation for `Odd` follows the same principle.

3. **Container Layer**. The container will store the state of all components and also implement all the interfaces provided by the components. The `Ctx` generic in the `Proxy` structure mentioned above will be instantiated as `Container` in this case:

   ```rust
   #[derive(Default)]
   struct Container {
       // If needed, these fields can be lazy-initialized.
       even_state: EvenState,
       odd_state: OddState,
   }
   
   // In order to enable XXProxy to dereference the component's state
   impl AsRef<EvenState> for Container { /**/ }
   impl AsMut<EvenState> for Container { /**/ }
   impl AsRef<OddState> for Container { /**/ }
   impl AsMut<OddState> for Container { /**/ }
   
   // Implement all the component interfaces for Container, allowing XXProxy to invoke them through the ctx
   impl Even for Container {
       #[inline]
       fn is_even(&mut self, n: u64) -> bool {
           // `&mut Container` -> `&mut EvenProxy<Container>`
           // When Proxy.ctx.is_even() is called, it will be forwarded by Container to the actual implementation.
           EvenProxy::ref_cast_mut(self).is_even(n)
       }
       
       #[inline]
       fn emit_count<T>(&self, f: FnOnce(usize) -> T) -> T {
           EvenProxy::ref_cast(self).emit_count(f)
       }
   }
   
   impl Odd for Container { /**/ }
   ```



Let's analyze the call chain of `EvenProxy::<Ctx>::is_even`:

1. `EvenProxy::<Ctx>::is_even` calls `Ctx::is_odd`.
2. `Ctx` is instantiated as `Container`, so it's equivalent to calling `Container::is_odd`.
3. `Container::is_odd` is actually dispatched to `OddProxy::<Container>::is_odd`.
4. `OddProxy::<Ctx>::is_odd` may internally call `Ctx::is_even`.
5. ...

The `Container` acts as an intermediary or mediator (similar  to the types in Middle Layer in previous methods 3 and 4). It is responsible for  dispatching the specific implementations.



Method 5 essentially solves most of the issues brought up by the  previous approaches. Currently, I haven't identified any other problems  except for the lack of support for other pointer types in `RefCast` (but those can be implemented independently).

* Supports inter-crate communication: achieved through the `Proxy` type.

* No limitations on interface form: trait definitions have minimal restrictions.
* Zero-cost interface calls: simple dispatch that can be inlined.
* Static interface provision: ability to statically check if `Ctx` satisfies a certain trait.
* Component state not exposed: the type of component state is exposed, but the internal structure can remain hidden.
* Component modularity: no Middle layer, components can be split.
* Fast compilation speed (relatively): no Middle layer, component  interface layer is separately defined. However, due to generics,  instantiation and code generation occur in the Container layer, which  may slow down compilation. Using `dyn Ctx` more in internal implementations can distribute compilation time across components.



***

You can refer to my repository for demo code: https://github.com/TOETOE55/dep-inj. In this repository, I have implemented some macros to generate template code related to `Proxy` for convenience.

```rust
#[derive(Default, dep_inj::DepInj)]
#[target(EvenProxy)] // generate `struct EventProxy`
pub struct EvenState {
    count: usize,
}

impl<Ctx: ?Sized> Even for EvenProxy<Ctx>
where
    Ctx: Odd,
{
    fn is_even(&mut self, n: u64) -> bool {
       is_even_impl(self, n)
    }
    
    fn emit_count<T>(&self, f: impl FnOnce(usize) -> T) -> T {
        f(self.count)
    }
}

trait EvenDeps: Odd + AsRef<EvenState> + AsMut<EvenState> {}
impl<T: Odd + AsRef<EvenState> + AsMut<EvenState>> EvenDeps for T {}

fn is_even_impl(
    proxy: &mut EvenProxy<dyn EvenDeps>,
    n: u64,
) -> bool {
    proxy.count += 1;

    (n == 0) || proxy
        .prj_ref_mut() 
        .is_odd(n - 1)
}
```

However, I'm currently not sure how to generate some of the code for `Container` in a more efficient way, so consider it as boilerplate for now.
