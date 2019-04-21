// -----------------------------------------------------------------------------

use std::process::{self, Command, Stdio};
use std::path::{self, Path};
use std::io::{self};
use std::thread::{self};
use std::collections::{self, HashSet};
use std::time::{self, Duration};
use chemfiles::{self, *};

// -----------------------------------------------------------------------------

pub type Position = [f64; 3];
pub type Velocity = [f64; 3];

// -----------------------------------------------------------------------------

#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub struct XYZAtom {
    pub name:            String,
    pub full_name:       String,
    pub atomic_type:     String,
    pub mass:            f64,
    pub charge:          f64,
    pub vdw_radius:      f64,
    pub covalent_radius: f64,
    pub atomic_number:   u64,
}

impl XYZAtom {
    pub fn new(string: &str) -> chemfiles::Result<Self> {
        let atom = Atom::new(string)?;
        let result = Self::from_atom(&atom)?;
        Ok(result)
    }

    pub fn from_atom(atom: &Atom) -> chemfiles::Result<Self> {
        Ok(XYZAtom {
            name:            atom.name()?,
            full_name:       atom.full_name()?,
            atomic_type:     atom.atomic_type()?,
            mass:            atom.mass()?,
            charge:          atom.charge()?,
            vdw_radius:      atom.vdw_radius()?,
            covalent_radius: atom.covalent_radius()?,
            atomic_number:   atom.atomic_number()?,
        })
    }

    pub fn to_atom(&self) -> chemfiles::Result<Atom> {
        let mut atom = Atom::new(self.name.as_str())?;
        atom.set_atomic_type(self.atomic_type.as_str())?;
        atom.set_mass(self.mass)?;
        atom.set_charge(self.charge)?;
        Ok(atom)
    }
}

// -----------------------------------------------------------------------------

#[derive(PartialEq, Clone, Debug)]
pub struct XYZUnitCell {
    pub matrix:  [[f64; 3]; 3],
    pub angles:  [f64; 3],
    pub lengths: [f64; 3],
    pub shape:   CellShape,
}

impl XYZUnitCell {
    pub fn new_infinite() -> chemfiles::Result<Self> {
        let cell = UnitCell::infinite()?;
        Self::from_unit_cell(&cell)
    }

    pub fn new_orthorhombic(lengths: [f64; 3]) -> chemfiles::Result<Self> {
        let cell = UnitCell::new(lengths)?;
        Self::from_unit_cell(&cell)
    }

    pub fn new_triclinic(lengths: [f64; 3], angles: [f64; 3])
                         -> chemfiles::Result<Self> {
        let cell = UnitCell::triclinic(lengths, angles)?;
        Self::from_unit_cell(&cell)
    }

    pub fn from_unit_cell(cell: &UnitCell) -> chemfiles::Result<Self> {
        Ok(XYZUnitCell {
            matrix:  cell.matrix()?,
            angles:  cell.angles()?,
            lengths: cell.lengths()?,
            shape:   cell.shape()?,
        })
    }

    pub fn to_unit_cell(&self) -> chemfiles::Result<UnitCell> {
        unimplemented!()
    }
}

// -----------------------------------------------------------------------------

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct XYZResidue {
    pub name:  String,
    pub id:    Option<u64>,
    pub atoms: HashSet<usize>,
}

impl XYZResidue {
    pub fn from_residue(natoms: usize, residue: &Residue)
                        -> chemfiles::Result<Self> {
        let name = residue.name()?;
        let id = residue.id()?;
        let atoms = {
            let mut set = HashSet::new();
            for i in 0 .. natoms {
                if residue.contains(i as u64)? {
                    set.insert(i);
                }
            }
            set
        };

        Ok(XYZResidue { name: name, id: id, atoms: atoms })
    }
}

// -----------------------------------------------------------------------------

#[derive(PartialEq, Clone, Debug)]
pub struct XYZTopology {
    pub atoms:     Vec<XYZAtom>,
    pub bonds:     Vec<[u64; 2]>,
    pub angles:    Vec<[u64; 3]>,
    pub dihedrals: Vec<[u64; 4]>,
    pub impropers: Vec<[u64; 4]>,
    pub residues:  Vec<XYZResidue>,
    pub linkage:   HashSet<(usize, usize)>,
}

impl XYZTopology {
    pub fn from_topology(topology: &Topology) -> chemfiles::Result<Self> {
        let atoms = {
            let mut vec = Vec::new();
            for i in 0 .. topology.size()? {
                vec.push(XYZAtom::from_atom(&(topology.atom(i)?))?);
            }
            vec
        };

        let residues = {
            let mut vec = Vec::new();
            for i in 0 .. topology.residues_count()? {
                vec.push(topology.residue(i)?);
            }
            vec
        };

        let converted_residues = {
            let natoms = topology.size()? as usize;
            let mut vec = Vec::new();
            for r in &residues {
                vec.push(XYZResidue::from_residue(natoms, r)?);
            }
            vec
        };

        let mut linkage = HashSet::new();
        for (i, a) in residues.iter().enumerate() {
            for (j, b) in residues.iter().enumerate() {
                if topology.are_linked(a, b)? {
                    linkage.insert((i, j));
                }
            }
        }

        Ok(XYZTopology {
            atoms:     atoms,
            bonds:     topology.bonds()?,
            angles:    topology.angles()?,
            dihedrals: topology.dihedrals()?,
            impropers: topology.impropers()?,
            residues:  converted_residues,
            linkage:   linkage,
        })
    }
}

// -----------------------------------------------------------------------------

#[derive(PartialEq, Clone, Debug)]
pub struct XYZState {
    pub step:        u64,
    pub atom_states: Vec<(XYZAtom, Position, Option<Velocity>)>,
    pub topology:    XYZTopology,
    pub cell:        XYZUnitCell,
}

impl XYZState {
    pub fn from_frame(frame: Frame) -> chemfiles::Result<Self> {
        let mut f = frame.clone();
        f.guess_topology()?;
        let step = f.step().unwrap();
        let atoms: Vec<Atom> = {
            let mut vec = Vec::new();
            for i in 0 .. f.size()? {
                vec.push(f.atom(i)?);
            }
            vec
        };
        let positions  = f.positions()?;
        let velocities = {
            if f.has_velocities()? {
                f.velocities()?.iter().cloned().map(Option::from).collect()
            } else {
                let mut vec = Vec::new();
                for _ in positions { vec.push(Option::None); }
                vec
            }
        };
        let cell = XYZUnitCell::from_unit_cell(&(f.cell()?))?;
        let topology = XYZTopology::from_topology(&(f.topology()?))?;
        let mut atom_states = Vec::new();
        for ((atom, pos), vel) in atoms.iter().zip(positions).zip(velocities) {
            atom_states.push(
                (XYZAtom::from_atom(atom)?, pos.clone(), vel.clone())
            );
        }
        Ok(XYZState {
            step:        step,
            atom_states: atom_states,
            topology:    topology,
            cell:        cell,
        })
    }
}

// -----------------------------------------------------------------------------

#[derive(PartialEq, Clone, Debug)]
pub struct XYZFile {
    states: Vec<XYZState>,
}

impl XYZFile {
    pub fn from_file(path: &Path) -> chemfiles::Result<Self> {
        let mut trajectory: Trajectory
            = Trajectory::open_with_format(path, 'r', "XYZ")?;
        Self::from_trajectory(&mut trajectory)
    }

    pub fn from_trajectory(traj: &mut Trajectory) -> chemfiles::Result<Self> {
        let mut frame = Frame::new()?;
        let mut states: Vec<XYZState> = Vec::new();
        loop {
            if let Err(e) = traj.read(&mut frame) {
                if e.status == Status::FileError { break; }
                return Err(e);
            }
            let state = XYZState::from_frame(frame.clone())?;
            states.push(state);
        }
        Ok(XYZFile { states: states })
    }
}

// -----------------------------------------------------------------------------

// #[cfg(test)]
pub mod tests {
    use super::*;

    // #[test]
    pub fn test_add() {
        let xyz_file = XYZFile::from_file(Path::new("/home/remy/Documents/NotWork/Projects/Rust/chemsim/cp2k/methane/methane.xyz")).unwrap();

        let mut temp: Option<XYZTopology> = Option::None;
        for state in xyz_file.states.iter().cycle() {
            // let atoms: Vec<Atom> = {
            //     let mut vec = Vec::new();
            //     for i in 0 .. frame.size().unwrap() {
            //         vec.push(frame.atom(i).unwrap());
            //     }
            //     vec
            // };
            // let positions = frame.positions().unwrap();
            // let velocities = frame.velocities().unwrap();
            // let cell = frame.cell().unwrap();
            // let topology = frame.topology().unwrap();

            println!("\x1Bc\n\x1Bc");
            println!("Step: {}", state.step);
            println!("  Atoms:     \x1B[1;33m{:?}\x1B[0m", state.topology.atoms.iter().map(|x| x.name.clone()).collect::<Vec<String>>());
            println!("  Bonds:     \x1B[1;33m{:?}\x1B[0m", state.topology.bonds);
            println!("  Angles:    \x1B[1;33m{:?}\x1B[0m", state.topology.angles);
            println!("  Dihedrals: \x1B[1;33m{:?}\x1B[0m", state.topology.dihedrals);
            println!("  Impropers: \x1B[1;33m{:?}\x1B[0m", state.topology.impropers);
            println!("  Residues:  \x1B[1;33m{:?}\x1B[0m", state.topology.residues);
            println!("  Linkage:   \x1B[1;33m{:?}\x1B[0m", state.topology.linkage);

            thread::sleep(Duration::from_millis(10));

            if temp != Some(state.topology.clone()) {
                thread::sleep(Duration::from_millis(300));
                temp = Some(state.topology.clone());
            }
        }
    }
}

// -----------------------------------------------------------------------------
