// ============================================================
//  Motor de Catálogo AVL — Biblioteca Municipal de Santa Ana
//  Materia: Estructuras de Datos / Programación II
// ============================================================

// --- ESTRUCTURAS BASE ---

#[derive(Debug, Clone)]
struct Libro {
    isbn: u32,
    titulo: String,
}

struct Nodo {
    libro: Libro,
    // Option<Box<Nodo>>:
    //   - Option  → el hijo puede no existir (None)
    //   - Box     → aloja el nodo hijo en el heap, porque Rust
    //               no permite tipos de tamaño desconocido en el stack.
    izquierdo: Option<Box<Nodo>>,
    derecho: Option<Box<Nodo>>,
    altura: i32,
}

impl Nodo {
    fn nuevo(libro: Libro) -> Self {
        Nodo {
            libro,
            izquierdo: None,
            derecho: None,
            altura: 1, // un nodo hoja siempre tiene altura 1
        }
    }
}

// ---------------------------------------------------------------
// FASE 1 — AUDITORÍA: funciones del código base comentadas
// ---------------------------------------------------------------

/// Devuelve la altura del nodo, o 0 si el nodo es None.
/// .as_ref() nos presta una referencia &Box<Nodo> sin consumir el Option,
/// map_or(0, |n| n.altura) aplica el closure si hay valor, 0 si no.
fn obtener_altura(nodo: &Option<Box<Nodo>>) -> i32 {
    nodo.as_ref().map_or(0, |n| n.altura)
}

/// Recalcula y actualiza la altura de un nodo a partir de sus hijos.
/// altura = 1 + max(altura_izq, altura_der)
fn actualizar_altura(nodo: &mut Nodo) {
    nodo.altura = 1 + std::cmp::max(
        obtener_altura(&nodo.izquierdo),
        obtener_altura(&nodo.derecho),
    );
}

/// Factor de balance: altura_izq − altura_der.
///  > 1  → el subárbol izquierdo es más alto (rotación a la derecha)
/// < -1  → el subárbol derecho  es más alto (rotación a la izquierda)
fn obtener_balance(nodo: &Nodo) -> i32 {
    obtener_altura(&nodo.izquierdo) - obtener_altura(&nodo.derecho)
}

// ---------------------------------------------------------------
// ANÁLISIS DE .take() (Fase 1, punto 3)
// ---------------------------------------------------------------
// .take() extrae el valor de un Option<T> dejando None en su lugar
// y transfiriendo la propiedad (ownership) al llamador.
// En las rotaciones necesitamos mover subárboles enteros (Box<Nodo>)
// de un campo a otro. Rust no permite mover un campo mientras el
// struct padre sigue existiendo (violación de borrow checker), porque
// dejaría el campo en un estado "parcialmente movido". .take() resuelve
// esto: reemplaza el campo con None de forma segura antes de mover el
// valor, garantizando que el struct padre siempre sea válido.
// Una asignación directa (nodo.izquierdo = ...) intentaría mover y
// escribir al mismo tiempo, lo que el compilador rechaza.

// ---------------------------------------------------------------
// ROTACIONES (código base, ahora comentado)
// ---------------------------------------------------------------

/// Rotación simple a la derecha sobre el nodo y.
///
///       y                x
///      / \              / \
///     x   T3    →     T1   y
///    / \                  / \
///   T1  T2              T2  T3
///
/// .take() se usa para mover x fuera de y.izquierdo sin
/// violar las reglas de ownership de Rust (ver análisis arriba).
fn rotar_derecha(mut y: Box<Nodo>) -> Box<Nodo> {
    // Extraemos x (hijo izquierdo de y) con .take()
    let mut x = y.izquierdo.take().expect("Hijo izquierdo ausente");
    // T2 (hijo derecho de x) pasa a ser hijo izquierdo de y
    y.izquierdo = x.derecho.take();
    actualizar_altura(&mut y);
    // y pasa a ser hijo derecho de x
    x.derecho = Some(y);
    actualizar_altura(&mut x);
    x // x es la nueva raíz de este subárbol
}

/// Rotación simple a la izquierda sobre el nodo x.
///
///     x                  y
///    / \                / \
///   T1  y      →      x   T3
///      / \           / \
///     T2  T3        T1  T2
fn rotar_izquierda(mut x: Box<Nodo>) -> Box<Nodo> {
    let mut y = x.derecho.take().expect("Hijo derecho ausente");
    x.derecho = y.izquierdo.take();
    actualizar_altura(&mut x);
    y.izquierdo = Some(x);
    actualizar_altura(&mut y);
    y
}

// ---------------------------------------------------------------
// INSERCIÓN + BALANCEO (código base, comentado)
// ---------------------------------------------------------------

/// Inserta un libro en el árbol AVL y retorna la nueva raíz balanceada.
/// Consume nodo_opt con .take() para poder pasar ownership a la llamada
/// recursiva sin dejar referencias colgantes.
fn insertar(nodo_opt: Option<Box<Nodo>>, libro: Libro) -> Box<Nodo> {
    let mut nodo = match nodo_opt {
        None => return Box::new(Nodo::nuevo(libro)), // árbol vacío → nuevo nodo
        Some(n) => n,
    };

    let isbn_nuevo = libro.isbn;

    if isbn_nuevo < nodo.libro.isbn {
        // Insertar en el subárbol izquierdo
        nodo.izquierdo = Some(insertar(nodo.izquierdo.take(), libro));
    } else if isbn_nuevo > nodo.libro.isbn {
        // Insertar en el subárbol derecho
        nodo.derecho = Some(insertar(nodo.derecho.take(), libro));
    } else {
        return nodo; // ISBN duplicado → no hacemos nada
    }

    actualizar_altura(&mut nodo);
    let balance = obtener_balance(&nodo);

    // Caso 1: desbalance izquierda-izquierda → rotación derecha simple
    if balance > 1 && isbn_nuevo < nodo.izquierdo.as_ref().unwrap().libro.isbn {
        return rotar_derecha(nodo);
    }
    // Caso 2: desbalance derecha-derecha → rotación izquierda simple
    if balance < -1 && isbn_nuevo > nodo.derecho.as_ref().unwrap().libro.isbn {
        return rotar_izquierda(nodo);
    }
    // Caso 3: desbalance izquierda-derecha → rotación doble (izq luego der)
    if balance > 1 && isbn_nuevo > nodo.izquierdo.as_ref().unwrap().libro.isbn {
        let hijo_izq = nodo.izquierdo.take().unwrap();
        nodo.izquierdo = Some(rotar_izquierda(hijo_izq));
        return rotar_derecha(nodo);
    }
    // Caso 4: desbalance derecha-izquierda → rotación doble (der luego izq)
    if balance < -1 && isbn_nuevo < nodo.derecho.as_ref().unwrap().libro.isbn {
        let hijo_der = nodo.derecho.take().unwrap();
        nodo.derecho = Some(rotar_derecha(hijo_der));
        return rotar_izquierda(nodo);
    }

    nodo // nodo ya balanceado
}

// ---------------------------------------------------------------
// FASE 2 — BÚSQUEDA POR ISBN
// ---------------------------------------------------------------

/// Busca un libro por ISBN en el árbol AVL.
/// Retorna Option<&Libro> — una referencia al libro si existe, None si no.
/// No copia datos: solo presta una referencia al libro ya almacenado.
/// El lifetime 'a garantiza que la referencia devuelta vive tanto
/// como el árbol que la contiene.
fn buscar<'a>(nodo: &'a Option<Box<Nodo>>, isbn: u32) -> Option<&'a Libro> {
    // as_ref() convierte &Option<Box<Nodo>> → Option<&Box<Nodo>>
    // sin consumir el Option original (solo prestamos, no movemos)
    match nodo.as_ref() {
        None => None, // árbol vacío o llegamos a una hoja
        Some(n) => {
            if isbn == n.libro.isbn {
                Some(&n.libro) // encontrado → devolvemos referencia
            } else if isbn < n.libro.isbn {
                buscar(&n.izquierdo, isbn) // buscar en subárbol izquierdo
            } else {
                buscar(&n.derecho, isbn)   // buscar en subárbol derecho
            }
        }
    }
}

// ---------------------------------------------------------------
// FASE 3 — ELIMINACIÓN CON RE-BALANCEO
// ---------------------------------------------------------------

/// Encuentra y extrae el nodo con el ISBN mínimo de un subárbol.
/// Se usa para obtener el sucesor in-orden al eliminar un nodo con dos hijos.
/// Retorna (nodo_minimo, arbol_restante_sin_ese_minimo).
fn extraer_minimo(nodo: Box<Nodo>) -> (Box<Nodo>, Option<Box<Nodo>>) {
    if nodo.izquierdo.is_none() {
        // Este nodo ES el mínimo; su hijo derecho queda en su lugar
        let derecho = nodo.derecho; // no usamos .take() porque consumimos nodo
        // Necesitamos separar derecho antes de mover nodo, así que
        // reconstruimos usando los campos:
        let min = Box::new(Nodo {
            libro: nodo.libro,
            izquierdo: None,
            derecho: None,
            altura: 1,
        });
        (min, derecho)
    } else {
        // Seguimos bajando por la izquierda
        let mut nodo = nodo;
        let izq = nodo.izquierdo.take().unwrap();
        let (min, nuevo_izq) = extraer_minimo(izq);
        nodo.izquierdo = nuevo_izq;
        actualizar_altura(&mut nodo);
        let nodo = balancear(nodo); // re-balancear tras la extracción
        (min, Some(nodo))
    }
}

/// Aplica las rotaciones AVL necesarias a un nodo ya con altura actualizada.
/// Función auxiliar usada tanto en eliminar como en extraer_minimo.
fn balancear(mut nodo: Box<Nodo>) -> Box<Nodo> {
    let balance = obtener_balance(&nodo);

    // Desbalance por la izquierda
    if balance > 1 {
        // Verificar si necesitamos rotación doble (izquierda-derecha)
        if obtener_balance(nodo.izquierdo.as_ref().unwrap()) < 0 {
            let hijo_izq = nodo.izquierdo.take().unwrap();
            nodo.izquierdo = Some(rotar_izquierda(hijo_izq));
        }
        return rotar_derecha(nodo);
    }

    // Desbalance por la derecha
    if balance < -1 {
        // Verificar si necesitamos rotación doble (derecha-izquierda)
        if obtener_balance(nodo.derecho.as_ref().unwrap()) > 0 {
            let hijo_der = nodo.derecho.take().unwrap();
            nodo.derecho = Some(rotar_derecha(hijo_der));
        }
        return rotar_izquierda(nodo);
    }

    nodo // ya estaba balanceado
}

/// Elimina el nodo con el ISBN dado del árbol AVL.
/// Maneja los 3 casos:
///   1. Nodo hoja (sin hijos)            → simplemente se elimina
///   2. Nodo con un solo hijo            → se reemplaza por ese hijo
///   3. Nodo con dos hijos               → se reemplaza por el sucesor in-orden
///      (el menor del subárbol derecho)
/// Tras eliminar, actualiza altura y re-balancea si es necesario.
fn eliminar(nodo_opt: Option<Box<Nodo>>, isbn: u32) -> Option<Box<Nodo>> {
    let mut nodo = match nodo_opt {
        None => return None, // ISBN no encontrado, no hay nada que eliminar
        Some(n) => n,
    };

    if isbn < nodo.libro.isbn {
        // El nodo a eliminar está en el subárbol izquierdo
        nodo.izquierdo = eliminar(nodo.izquierdo.take(), isbn);
    } else if isbn > nodo.libro.isbn {
        // El nodo a eliminar está en el subárbol derecho
        nodo.derecho = eliminar(nodo.derecho.take(), isbn);
    } else {
        // ¡Encontramos el nodo a eliminar!
        match (nodo.izquierdo.take(), nodo.derecho.take()) {
            // Caso 1: nodo hoja — sin hijos
            (None, None) => return None,

            // Caso 2a: solo tiene hijo derecho
            (None, Some(der)) => return Some(der),

            // Caso 2b: solo tiene hijo izquierdo
            (Some(izq), None) => return Some(izq),

            // Caso 3: tiene dos hijos → usar sucesor in-orden
            (Some(izq), Some(der)) => {
                // Extraemos el sucesor in-orden (mínimo del subárbol derecho)
                let (sucesor, nuevo_der) = extraer_minimo(der);
                // Reconstruimos el nodo con los datos del sucesor
                nodo.libro = sucesor.libro;
                nodo.izquierdo = Some(izq);
                nodo.derecho = nuevo_der;
            }
        }
    }

    // Actualizar altura y re-balancear antes de retornar
    actualizar_altura(&mut nodo);
    Some(balancear(nodo))
}

// ---------------------------------------------------------------
// FASE 4 — OPCIÓN B: ESTADÍSTICAS DEL ÁRBOL
// ---------------------------------------------------------------

/// Estructura para acumular las estadísticas del árbol.
struct Estadisticas {
    total_nodos: u32,
    altura_total: i32,
    isbn_mas_alto: Option<u32>,
    titulo_isbn_mas_alto: Option<String>,
}

/// Recorre el árbol en in-orden y acumula estadísticas:
///   - total_nodos: cantidad de libros en el catálogo
///   - altura_total: altura del árbol (profundidad máxima)
///   - isbn_mas_alto: el ISBN de mayor valor (nodo más a la derecha)
fn calcular_estadisticas(nodo: &Option<Box<Nodo>>, stats: &mut Estadisticas, profundidad_actual: i32) {
    if let Some(n) = nodo.as_ref() {
        stats.total_nodos += 1;

        // La altura total es la mayor profundidad alcanzada
        if profundidad_actual > stats.altura_total {
            stats.altura_total = profundidad_actual;
        }

        // El ISBN más alto siempre está en el extremo derecho del árbol
        match stats.isbn_mas_alto {
            None => {
                stats.isbn_mas_alto = Some(n.libro.isbn);
                stats.titulo_isbn_mas_alto = Some(n.libro.titulo.clone());
            }
            Some(actual) if n.libro.isbn > actual => {
                stats.isbn_mas_alto = Some(n.libro.isbn);
                stats.titulo_isbn_mas_alto = Some(n.libro.titulo.clone());
            }
            _ => {}
        }

        calcular_estadisticas(&n.izquierdo, stats, profundidad_actual + 1);
        calcular_estadisticas(&n.derecho,   stats, profundidad_actual + 1);
    }
}

// ---------------------------------------------------------------
// IMPRESIÓN (código base, sin cambios)
// ---------------------------------------------------------------

/// Imprime el árbol rotado 90° (derecha arriba, izquierda abajo).
/// La indentación representa la profundidad del nodo.
fn imprimir(nodo: &Option<Box<Nodo>>, nivel: usize) {
    if let Some(n) = nodo {
        imprimir(&n.derecho, nivel + 1);
        println!("{:indent$}[ISBN: {}] {}", "", n.libro.isbn, n.libro.titulo, indent = nivel * 4);
        imprimir(&n.izquierdo, nivel + 1);
    }
}

// ---------------------------------------------------------------
// MAIN — PRUEBAS COMPLETAS
// ---------------------------------------------------------------

fn main() {
    // ── Construcción del árbol ──────────────────────────────────
    let mut raiz: Option<Box<Nodo>> = None;

    let datos = vec![
        (10, "El Quijote"),
        (20, "1984"),
        (30, "Hamlet"),
        (5,  "Fahrenheit 451"),
        (2,  "La Odisea"),
        (25, "El Principito"),
    ];

    println!("======================================================");
    println!("  Sistema de Inventario de Librería — Árbol AVL");
    println!("======================================================\n");

    println!(">>> Insertando libros en orden: 10, 20, 30, 5, 2, 25");
    println!("    (el balanceo AVL se aplica automáticamente)\n");

    for (isbn, titulo) in datos {
        let libro = Libro { isbn, titulo: titulo.to_string() };
        raiz = Some(insertar(raiz.take(), libro));
    }

    println!("--- Árbol después de todas las inserciones ---");
    println!("    (lectura: derecha = arriba, izquierda = abajo)\n");
    imprimir(&raiz, 0);

    // ── FASE 1: Prueba de escritorio ────────────────────────────
    println!("\n--- Prueba de escritorio: secuencia de inserciones ---");
    println!("Insertar 10 → raíz=[10]");
    println!("Insertar 20 → [10]-der=[20]");
    println!("Insertar 30 → balance(10)=-2 *** ROTACIÓN IZQUIERDA *** → raíz=[20], izq=[10], der=[30]");
    println!("Insertar  5 → [10]-izq=[5]");
    println!("Insertar  2 → balance(10)=2  *** ROTACIÓN DERECHA ***   → [5] sube, izq=[2], der=[10]");
    println!("Insertar 25 → [30]-izq=[25], balance ok");
    println!("Estado final: raíz=20, izq=5(izq=2,der=10), der=30(izq=25)");

    // ── FASE 2: Búsqueda ────────────────────────────────────────
    println!("\n======================================================");
    println!("  FASE 2 — Búsqueda por ISBN");
    println!("======================================================\n");

    let isbn_buscar = 25;
    match buscar(&raiz, isbn_buscar) {
        Some(libro) => println!("[OK] ISBN {} encontrado → \"{}\"", libro.isbn, libro.titulo),
        None        => println!("[--] ISBN {} no encontrado en el catálogo.", isbn_buscar),
    }

    let isbn_inexistente = 99;
    match buscar(&raiz, isbn_inexistente) {
        Some(libro) => println!("[OK] ISBN {} encontrado → \"{}\"", libro.isbn, libro.titulo),
        None        => println!("[--] ISBN {} no existe en el catálogo (esperado).", isbn_inexistente),
    }

    // Búsqueda en árbol vacío (caso borde)
    let arbol_vacio: Option<Box<Nodo>> = None;
    match buscar(&arbol_vacio, 10) {
        Some(_) => println!("[OK] Encontrado en árbol vacío (no debería pasar)"),
        None    => println!("[--] Búsqueda en árbol vacío → None (correcto)."),
    }

    // ── FASE 3: Eliminación ─────────────────────────────────────
    println!("\n======================================================");
    println!("  FASE 3 — Eliminación con re-balanceo");
    println!("======================================================\n");

    // Caso 1: eliminar nodo hoja (ISBN 2)
    println!(">> Eliminando ISBN 2 (nodo hoja)...");
    raiz = eliminar(raiz.take(), 2);
    imprimir(&raiz, 0);

    // Caso 2: eliminar nodo con un hijo (ISBN 10 tiene solo hijo derecho si 2 ya se fue... o probar con otro)
    println!("\n>> Eliminando ISBN 5 (nodo con un hijo: solo derecho=10)...");
    raiz = eliminar(raiz.take(), 5);
    imprimir(&raiz, 0);

    // Caso 3: eliminar nodo con dos hijos (ISBN 20, la raíz actual)
    println!("\n>> Eliminando ISBN 20 (nodo con dos hijos — sucesor in-orden=25)...");
    raiz = eliminar(raiz.take(), 20);
    imprimir(&raiz, 0);

    // Eliminar ISBN inexistente (no debe romper nada)
    println!("\n>> Eliminando ISBN 999 (no existe — no debe cambiar el árbol)...");
    raiz = eliminar(raiz.take(), 999);
    imprimir(&raiz, 0);

    // ── FASE 4: Estadísticas ────────────────────────────────────
    println!("\n======================================================");
    println!("  FASE 4 — Estadísticas del catálogo");
    println!("======================================================\n");

    // Reconstruimos el árbol completo para las estadísticas
    let mut raiz2: Option<Box<Nodo>> = None;
    let datos2 = vec![
        (10, "El Quijote"), (20, "1984"), (30, "Hamlet"),
        (5, "Fahrenheit 451"), (2, "La Odisea"), (25, "El Principito"),
    ];
    for (isbn, titulo) in datos2 {
        raiz2 = Some(insertar(raiz2.take(), Libro { isbn, titulo: titulo.to_string() }));
    }

    let mut stats = Estadisticas {
        total_nodos: 0,
        altura_total: 0,
        isbn_mas_alto: None,
        titulo_isbn_mas_alto: None,
    };
    calcular_estadisticas(&raiz2, &mut stats, 1);

    println!("  Total de libros en catálogo : {}", stats.total_nodos);
    println!("  Altura del árbol AVL        : {}", stats.altura_total);
    match (stats.isbn_mas_alto, stats.titulo_isbn_mas_alto) {
        (Some(isbn), Some(titulo)) =>
            println!("  Libro con ISBN más alto     : ISBN {} — \"{}\"", isbn, titulo),
        _ =>
            println!("  El catálogo está vacío."),
    }

    println!("\n======================================================");
    println!("  Fin del programa");
    println!("======================================================");
}
