pub struct Patient {
    pub name: String,
    pub id: u32,
}

pub fn process_patient(patient: Patient) {
    // GhostHealth Guard should catch this PHI leak!
    println!("DEBUG: Processing patient name: {}", patient.name);
}