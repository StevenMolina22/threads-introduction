use std::thread;
use std::time::Duration;

/// Los estados por los que puede pasar una tarea.
///
/// La ventaja del enum es que Rust nos obliga a pensar qué hacemos en cada caso
/// cuando usamos `match`. Si mañana agregamos otro estado, el compilador nos va
/// a avisar dónde falta contemplarlo.
#[derive(Debug, Clone, PartialEq, Eq)]
enum EstadoTarea {
    Pendiente,
    EnProgreso,
    Completada,
    Fallida(String),
}

/// Una tarea simple para usar en todo el ejemplo.
///
/// La idea es que tenga datos chicos, como el `id`, y datos dinámicos, como la
/// `descripcion`, para poder hablar de stack, heap, ownership y concurrencia con
/// el mismo modelo.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Tarea {
    id: u32,
    descripcion: String,
    estado: EstadoTarea,
}

impl Tarea {
    /// Crea una tarea nueva. Por defecto arranca pendiente.
    fn new(id: u32, descripcion: impl Into<String>) -> Self {
        Self {
            id,
            descripcion: descripcion.into(),
            estado: EstadoTarea::Pendiente,
        }
    }

    /// Consulta el estado actual sin modificar la tarea.
    fn obtener_estado(&self) -> &EstadoTarea {
        &self.estado
    }

    /// Marca la tarea como terminada.
    fn completar(&mut self) {
        self.estado = EstadoTarea::Completada;
    }

    /// Guarda que la tarea falló junto con una explicación breve.
    fn fallar(&mut self, mensaje: impl Into<String>) {
        self.estado = EstadoTarea::Fallida(mensaje.into());
    }
}

/// Algo que puede procesarse.
///
/// En Rust este tipo de comportamiento se expresa con traits. Es parecido a
/// decir: "si un tipo implementa este trait, entonces sabe ejecutarse". No hace
/// falta armar una jerarquía de clases para compartir esta capacidad.
trait Procesable {
    fn ejecutar(&mut self) -> Result<(), String>;
}

impl Procesable for Tarea {
    fn ejecutar(&mut self) -> Result<(), String> {
        self.estado = EstadoTarea::EnProgreso;

        if self.descripcion.trim().is_empty() {
            let mensaje = format!("La tarea {} quedó sin descripción", self.id);
            self.fallar(mensaje.clone());
            Err(mensaje)
        } else if self.descripcion.to_lowercase().contains("fallar") {
            let mensaje = format!("La tarea {} encontró un problema al procesarse", self.id);
            self.fallar(mensaje.clone());
            Err(mensaje)
        } else {
            self.completar();
            Ok(())
        }
    }
}

fn main() {
    println!("=== Ejemplo completo: gestor de tareas en Rust ===\n");

    demostrar_variables_y_memoria();

    let mut tareas = crear_tareas_de_ejemplo();
    mostrar_tareas("Así arrancan las tareas", &tareas);

    println!("\n--- Buscando una tarea con Option<Tarea> ---");
    match buscar_tarea(2, &tareas) {
        Some(tarea) => println!("Encontré esta tarea: {:?}", tarea),
        None => println!("No encontré una tarea con ese id."),
    }

    println!("\n--- Revisando estados con match ---");
    for tarea in &tareas {
        manejar_estado(tarea.obtener_estado());
    }

    println!("\n--- Procesando tareas con Result<(), String> ---");
    for tarea in &mut tareas {
        match tarea.ejecutar() {
            Ok(()) => println!("La tarea {} salió bien.", tarea.id),
            Err(error) => println!("La tarea {} no pudo terminar: {}", tarea.id, error),
        }
    }
    mostrar_tareas("Así quedan después de procesarlas", &tareas);

    println!("\n--- Ownership y borrowing ---");
    let tarea_para_analizar = Tarea::new(99, "Mirar una tarea sin quedarnos con ella");
    analizar_tarea(&tarea_para_analizar);
    println!(
        "Después de prestarla, todavía puedo usarla acá en main. id={}",
        tarea_para_analizar.id
    );

    println!("\n--- Mandando una tarea a otro hilo ---");
    let tarea_background = Tarea::new(100, "Procesar algo en segundo plano");
    procesar_en_hilo(tarea_background);

    println!("\n=== Fin del recorrido ===");
}

/// Muestra, con un loop chico, qué variables cambian y cuáles no.
/// También deja a la vista la diferencia entre un valor simple en stack y un
/// texto dinámico manejado en heap.
fn demostrar_variables_y_memoria() {
    println!("--- Variables, mutabilidad, stack y heap ---");

    let max_tareas: u32 = 4; // No cambia: es el límite de esta mini simulación.
    let mut procesadas: u32 = 0; // Cambia en cada vuelta del loop.

    while procesadas < max_tareas {
        // Un `u32` tiene tamaño fijo, así que es un buen ejemplo de dato en stack.
        let task_id: u32 = procesadas + 1;

        // Un `String` puede crecer mientras corre el programa, por eso usa heap.
        let mut task_description = String::from("Tarea");
        task_description.push_str(&format!(" #{}: leyendo datos del servidor", task_id));

        println!(
            "[Gestor] id={} (stack) | descripción='{}' (heap)",
            task_id, task_description
        );

        procesadas += 1;
    }

    println!("Listo: se simularon {}/{} tareas.", procesadas, max_tareas);
}

fn crear_tareas_de_ejemplo() -> Vec<Tarea> {
    vec![
        Tarea::new(1, "Aprender Rust"),
        Tarea::new(2, "Escribir el informe"),
        Tarea::new(3, "Esta tarea va a fallar a propósito"),
    ]
}

fn mostrar_tareas(titulo: &str, tareas: &[Tarea]) {
    println!("\n--- {} ---", titulo);
    for tarea in tareas {
        println!("{:?}", tarea);
    }
}

/// Busca una tarea por id.
///
/// Si la encuentra devuelve `Some(tarea)`. Si no está, devuelve `None`. En este
/// caso no lo tratamos como error: simplemente puede pasar que no exista.
fn buscar_tarea(id: u32, tareas: &[Tarea]) -> Option<Tarea> {
    tareas.iter().find(|tarea| tarea.id == id).cloned()
}

/// Decide qué mensaje mostrar según el estado actual.
fn manejar_estado(estado: &EstadoTarea) {
    match estado {
        EstadoTarea::Pendiente => {
            println!("Todavía está pendiente, esperando su turno.");
        }
        EstadoTarea::EnProgreso => {
            println!("Está en marcha ahora mismo.");
        }
        EstadoTarea::Completada => {
            println!("Terminó correctamente.");
        }
        EstadoTarea::Fallida(motivo) => {
            println!("Falló y habría que revisarla. Motivo: {motivo}");
        }
    }
}

/// Mira una tarea sin quedarse con ella.
///
/// Recibir `&Tarea` significa que la función solo toma prestada la tarea. Cuando
/// termina, quien la llamó sigue siendo el dueño y puede seguir usándola.
fn analizar_tarea(tarea: &Tarea) {
    println!(
        "La miro por referencia: id={}, descripción='{}', estado={:?}",
        tarea.id, tarea.descripcion, tarea.estado
    );
}

/// Procesa una tarea en un hilo aparte.
///
/// El `move` hace que el hilo se quede con la tarea. Así evitamos que el hilo
/// principal y el secundario intenten usar el mismo dato sin coordinación.
fn procesar_en_hilo(mut tarea: Tarea) {
    let manejador = thread::spawn(move || {
        println!("[Hilo] Arranco con la tarea {}...", tarea.id);
        thread::sleep(Duration::from_millis(200));

        match tarea.ejecutar() {
            Ok(()) => println!("[Hilo] Tarea {} lista: {:?}", tarea.id, tarea.estado),
            Err(error) => println!("[Hilo] Tarea {} terminó con error: {}", tarea.id, error),
        }
    });

    println!("[Main] Ya la mandé al hilo. Ahora espero a que termine...");
    manejador
        .join()
        .expect("el hilo secundario no debería fallar");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tarea_nueva_comienza_pendiente() {
        let tarea = Tarea::new(1, "Test");

        assert_eq!(tarea.obtener_estado(), &EstadoTarea::Pendiente);
    }

    #[test]
    fn completar_cambia_el_estado_a_completada() {
        let mut tarea = Tarea::new(1, "Test");

        tarea.completar();

        assert_eq!(tarea.estado, EstadoTarea::Completada);
    }

    #[test]
    fn buscar_tarea_existente_devuelve_some() {
        let tareas = crear_tareas_de_ejemplo();

        let encontrada = buscar_tarea(2, &tareas);

        assert_eq!(
            encontrada.map(|tarea| tarea.descripcion),
            Some("Escribir el informe".to_string())
        );
    }

    #[test]
    fn buscar_tarea_inexistente_devuelve_none() {
        let tareas = crear_tareas_de_ejemplo();

        let encontrada = buscar_tarea(999, &tareas);

        assert_eq!(encontrada, None);
    }

    #[test]
    fn ejecutar_tarea_valida_devuelve_ok_y_completa() {
        let mut tarea = Tarea::new(1, "Procesar datos");

        let resultado = tarea.ejecutar();

        assert_eq!(resultado, Ok(()));
        assert_eq!(tarea.estado, EstadoTarea::Completada);
    }

    #[test]
    fn ejecutar_tarea_invalida_devuelve_error_y_falla() {
        let mut tarea = Tarea::new(1, "");

        let resultado = tarea.ejecutar();

        assert!(resultado.is_err());
        assert!(matches!(tarea.estado, EstadoTarea::Fallida(_)));
    }
}
