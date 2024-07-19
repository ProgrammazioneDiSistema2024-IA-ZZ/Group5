use device_query::{DeviceQuery, DeviceState, MouseState};

fn main() {
    let device_state = DeviceState::new();

    let mut is_drawing = false;
    let mut start_position = (0, 0);
    let mut end_position: (i32, i32);

    loop {
        let mouse: MouseState = device_state.get_mouse();
        let position = mouse.coords;

        if mouse.button_pressed[1] {
            if !is_drawing {
                is_drawing = true;
                start_position = position;
                println!("Start drawing at {:?}", start_position);
            } else {
                end_position = position;
                println!("Drawing... Current position: {:?}", end_position);
            }
        } else {
            if is_drawing {
                is_drawing = false;
                end_position = position;
                println!("Finished drawing. Start: {:?}, End: {:?}", start_position, end_position);

                //L'utente schiaccia il tasto sinistro del mouse e disegna un lato del rettangolo. Rilascia il tasto, lo schiaccia di nuovo e disegna il secondo lato del rettangolo. Così via fino a che il rettangolo non è completo
                //In questo modo, c'è meno rischio che venga disegnato un rettangolo "accidentalmente", durante le normali operazioni al PC.
                //TODO: scrivere la logica per riconoscere quando un rettangolo viene disegnato
            }
        }
    }
}