import copy
import json
import subprocess
from pathlib import Path


def generate_simcells_netlist(simcells_verilog: Path, netlist: Path):
    passes = f"""
        read_verilog -sv -icells -lib {simcells_verilog};
        proc;
        clean;
        autoname;
        setundef -zero;
        flatten -scopename;
        write_json {netlist};
    """
    args = ["yosys", "-p", passes]

    result = subprocess.run(
        args, capture_output=True, timeout=60, text=True, encoding="utf-8"
    )
    assert result.returncode == 0, result.stderr


def clean_simcells_netlist(netlist_path: Path, new_netlist_path: Path):
    with open(netlist_path, "r") as f:
        netlist = json.load(f)

    new_netlist = {k: v for k, v in netlist.items() if k != "modules"}
    new_netlist["modules"] = {}

    for module_name, module in netlist["modules"].items():
        new_module_name = f"{module_name}_WRAPPER"
        new_netlist["modules"][new_module_name] = copy.deepcopy(module)

        if len(module["cells"]) > 0:
            continue

        module_cell = {
            "type": module_name,
            "port_directions": {},
            "connections": {},
        }

        for port_name, port_info in module["ports"].items():
            # print(port_info)
            module_cell["port_directions"][port_name] = port_info["direction"]
            module_cell["connections"][port_name] = port_info["bits"]

        new_netlist["modules"][new_module_name]["cells"] = {module_name: module_cell}

    with open(new_netlist_path, "w") as f:
        json.dump(new_netlist, f)

    # print(netlist)


if __name__ == "__main__":
    rtl_path = Path("/opt/oss-cad-suite/latest/share/yosys/simcells.v")
    netlist_path = Path("./deps/simcells_wrappers.json")

    generate_simcells_netlist(rtl_path, netlist_path)
    clean_simcells_netlist(netlist_path, netlist_path)
