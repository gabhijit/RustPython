/*
 * Dynamic type creation and names for built in types.
 */

use super::super::obj::{objstr, objtuple, objtype};
use super::super::pyobject::{
    DictProtocol, PyContext, PyFuncArgs, PyObjectRef, PyResult, TypeProtocol,
};
use super::super::VirtualMachine;

fn types_new_class(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(
        vm,
        args,
        required = [(name, Some(vm.ctx.str_type()))],
        optional = [(bases, None), (_kwds, None), (_exec_body, None)]
    );

    let name = objstr::get_value(name);
    let dict = vm.ctx.new_dict();

    let bases = match bases {
        Some(b) => {
            if objtype::isinstance(b, vm.ctx.tuple_type()) {
                objtuple::get_elements(b)
            } else {
                return Err(vm.new_type_error("Bases must be a tuple".to_string()));
            }
        }
        None => vec![vm.ctx.object()],
    };

    objtype::new(vm.ctx.type_type(), &name, bases, dict)
}

pub fn mk_module(ctx: &PyContext) -> PyObjectRef {
    let py_mod = ctx.new_module(&"types".to_string(), ctx.new_scope(None));

    // Number theory functions:
    py_mod.set_item("new_class", ctx.new_rustfunc(types_new_class));
    py_mod.set_item("FunctionType", ctx.function_type());

    py_mod
}
