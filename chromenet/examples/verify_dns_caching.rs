use chromenet::dns::{HickoryResolver, Resolve, Name};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resolver = HickoryResolver::new();
    let name = Name::new("google.com");
    
    println!("=== First lookup (cold) ===");
    let start = Instant::now();
    let _ = resolver.resolve(name.clone()).await?;
    println!("Time: {:?}", start.elapsed());
    
    // Sleep briefly to ensure it's not just a tight loop optimization
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    println!("\n=== Second lookup (simulated warm) ===");
    let start = Instant::now();
    let _ = resolver.resolve(name).await?;
    println!("Time: {:?}", start.elapsed());
    
    Ok(())
}
