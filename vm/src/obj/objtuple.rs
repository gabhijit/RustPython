use super::super::pyobject::{
    AttributeProtocol, PyContext, PyFuncArgs, PyObjectKind, PyObjectRef, PyResult, TypeProtocol,
};
use super::super::vm::VirtualMachine;
use super::objsequence::seq_equal;
use super::objstr;
use super::objtype;

fn tuple_eq(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(
        vm,
        args,
        required = [(zelf, Some(vm.ctx.tuple_type())), (other, None)]
    );

    let result = if objtype::isinstance(other, vm.ctx.tuple_type()) {
        let zelf = get_elements(zelf);
        let other = get_elements(other);
        seq_equal(vm, zelf, other)?
    } else {
        false
    };
    Ok(vm.ctx.new_bool(result))
}

fn tuple_len(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(vm, args, required = [(zelf, Some(vm.ctx.tuple_type()))]);
    let elements = get_elements(zelf);
    Ok(vm.context().new_int(elements.len() as i32))
}

fn tuple_repr(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(vm, args, required = [(zelf, Some(vm.ctx.tuple_type()))]);

    let elements = get_elements(zelf);

    let mut str_parts = vec![];
    for elem in elements {
        match vm.to_repr(elem) {
            Ok(s) => str_parts.push(objstr::get_value(&s)),
            Err(err) => return Err(err),
        }
    }

    let s = if str_parts.len() == 1 {
        format!("({},)", str_parts[0])
    } else {
        format!("({})", str_parts.join(", "))
    };
    Ok(vm.new_str(s))
}

pub fn get_elements(obj: &PyObjectRef) -> Vec<PyObjectRef> {
    if let PyObjectKind::Tuple { elements } = &obj.borrow().kind {
        elements.to_vec()
    } else {
        panic!("Cannot extract elements from non-tuple");
    }
}

pub fn init(context: &PyContext) {
    let ref tuple_type = context.tuple_type;
    tuple_type.set_attr("__eq__", context.new_rustfunc(tuple_eq));
    tuple_type.set_attr("__len__", context.new_rustfunc(tuple_len));
    tuple_type.set_attr("__repr__", context.new_rustfunc(tuple_repr));
}
