use crate::{Bounds, InputBounds};
use crate::model::common::Anchor;

/// LayoutDimensions - Centralizes ALL layout and positioning mathematical operations
///
/// Eliminates ad hoc layout math like:
/// - percentage parsing and normalization
/// - bounds calculations from InputBounds
/// - percentage.clamp(0.0, 100.0) / 100.0
/// - (normalized * (total - 1) as f64).round() as usize
/// - Anchor resolution and positioning
///
/// Replaces scattered layout logic from utils.rs, model/common.rs, and layout calculation code.
#[derive(Debug, Clone)]
pub struct LayoutDimensions {
    /// Total available space for layout
    total_bounds: Bounds,
    /// Child constraints and positioning
    child_constraints: Vec<InputBounds>,
    /// Layout strategy (how children are arranged)
    layout_strategy: LayoutStrategy,
}

impl LayoutDimensions {
    /// Create new layout dimensions
    pub fn new(total_bounds: Bounds) -> Self {
        Self {
            total_bounds,
            child_constraints: Vec::new(),
            layout_strategy: LayoutStrategy::Absolute,
        }
    }
    
    /// Add child constraint
    pub fn add_child(&mut self, input_bounds: InputBounds) {
        self.child_constraints.push(input_bounds);
    }
    
    /// Set layout strategy
    pub fn with_strategy(mut self, strategy: LayoutStrategy) -> Self {
        self.layout_strategy = strategy;
        self
    }
    
    /// Parse percentage string to normalized value (0.0-1.0)
    /// Centralizes: percentage.clamp(0.0, 100.0) / 100.0
    pub fn parse_percentage(percentage_str: &str) -> Result<f64, LayoutError> {
        if let Some(percent_part) = percentage_str.strip_suffix('%') {
            match percent_part.parse::<f64>() {
                Ok(value) => Ok(value.clamp(0.0, 100.0) / 100.0),
                Err(_) => Err(LayoutError::InvalidPercentage(percentage_str.to_string())),
            }
        } else {
            Err(LayoutError::InvalidPercentage(percentage_str.to_string()))
        }
    }
    
    /// Convert percentage to absolute coordinate within total bounds
    /// Centralizes: (normalized * (total - 1) as f64).round() as usize
    pub fn percentage_to_absolute(&self, percentage: f64, axis: Axis) -> usize {
        let total = match axis {
            Axis::X => self.total_bounds.width(),
            Axis::Y => self.total_bounds.height(),
        };
        
        if total <= 1 {
            return 0;
        }
        
        let normalized = percentage.clamp(0.0, 1.0);
        ((normalized * (total - 1) as f64).round() as usize).min(total - 1)
    }
    
    /// Parse InputBounds into actual Bounds
    /// Centralizes all InputBounds → Bounds conversion logic from utils.rs
    pub fn resolve_input_bounds(&self, input: &InputBounds) -> Result<Bounds, LayoutError> {
        let base = &self.total_bounds;
        
        let x1 = if input.x1.is_empty() {
            base.x1
        } else if input.x1.ends_with('%') {
            let percent = Self::parse_percentage(&input.x1)?;
            base.x1 + self.percentage_to_absolute(percent, Axis::X)
        } else {
            base.x1 + input.x1.parse::<usize>().map_err(|_| LayoutError::InvalidCoordinate(input.x1.clone()))?
        };
        
        let y1 = if input.y1.is_empty() {
            base.y1
        } else if input.y1.ends_with('%') {
            let percent = Self::parse_percentage(&input.y1)?;
            base.y1 + self.percentage_to_absolute(percent, Axis::Y)
        } else {
            base.y1 + input.y1.parse::<usize>().map_err(|_| LayoutError::InvalidCoordinate(input.y1.clone()))?
        };
        
        let x2 = if input.x2.is_empty() {
            base.x2
        } else if input.x2.ends_with('%') {
            let percent = Self::parse_percentage(&input.x2)?;
            base.x1 + self.percentage_to_absolute(percent, Axis::X)
        } else {
            base.x1 + input.x2.parse::<usize>().map_err(|_| LayoutError::InvalidCoordinate(input.x2.clone()))?
        };
        
        let y2 = if input.y2.is_empty() {
            base.y2
        } else if input.y2.ends_with('%') {
            let percent = Self::parse_percentage(&input.y2)?;
            base.y1 + self.percentage_to_absolute(percent, Axis::Y)
        } else {
            base.y1 + input.y2.parse::<usize>().map_err(|_| LayoutError::InvalidCoordinate(input.y2.clone()))?
        };
        
        Ok(Bounds::new(x1, y1, x2, y2))
    }
    
    /// Resolve anchor positioning for a target size within bounds
    /// Centralizes anchor resolution logic
    pub fn resolve_anchor(
        &self,
        anchor: &Anchor,
        target_size: (usize, usize),
        container: &Bounds,
    ) -> Bounds {
        let (target_width, target_height) = target_size;
        let container_width = container.width();
        let container_height = container.height();
        
        // Calculate position based on anchor
        let (x1, y1) = match anchor {
            Anchor::TopLeft => (container.x1, container.y1),
            Anchor::CenterTop => (
                container.x1 + container_width.saturating_sub(target_width) / 2,
                container.y1,
            ),
            Anchor::TopRight => (
                container.x1 + container_width.saturating_sub(target_width),
                container.y1,
            ),
            Anchor::CenterLeft => (
                container.x1,
                container.y1 + container_height.saturating_sub(target_height) / 2,
            ),
            Anchor::Center => (
                container.x1 + container_width.saturating_sub(target_width) / 2,
                container.y1 + container_height.saturating_sub(target_height) / 2,
            ),
            Anchor::CenterRight => (
                container.x1 + container_width.saturating_sub(target_width),
                container.y1 + container_height.saturating_sub(target_height) / 2,
            ),
            Anchor::BottomLeft => (
                container.x1,
                container.y1 + container_height.saturating_sub(target_height),
            ),
            Anchor::CenterBottom => (
                container.x1 + container_width.saturating_sub(target_width) / 2,
                container.y1 + container_height.saturating_sub(target_height),
            ),
            Anchor::BottomRight => (
                container.x1 + container_width.saturating_sub(target_width),
                container.y1 + container_height.saturating_sub(target_height),
            ),
        };
        
        Bounds::new(x1, y1, x1 + target_width - 1, y1 + target_height - 1)
    }
    
    /// Calculate all child bounds based on constraints and strategy
    pub fn calculate_all_child_bounds(&self) -> Result<Vec<Bounds>, LayoutError> {
        match &self.layout_strategy {
            LayoutStrategy::Absolute => self.calculate_absolute_layout(),
            LayoutStrategy::Grid { columns, rows } => self.calculate_grid_layout(*columns, *rows),
            LayoutStrategy::Flex { direction } => self.calculate_flex_layout(*direction),
            LayoutStrategy::Stack => self.calculate_stack_layout(),
        }
    }
    
    /// Calculate absolute positioning layout
    fn calculate_absolute_layout(&self) -> Result<Vec<Bounds>, LayoutError> {
        let mut results = Vec::new();
        
        for constraint in &self.child_constraints {
            let bounds = self.resolve_input_bounds(constraint)?;
            results.push(bounds);
        }
        
        Ok(results)
    }
    
    /// Calculate grid-based layout
    fn calculate_grid_layout(&self, columns: usize, rows: usize) -> Result<Vec<Bounds>, LayoutError> {
        if columns == 0 || rows == 0 {
            return Err(LayoutError::InvalidGridDimensions { columns, rows });
        }
        
        let cell_width = self.total_bounds.width() / columns;
        let cell_height = self.total_bounds.height() / rows;
        let mut results = Vec::new();
        
        for (index, _) in self.child_constraints.iter().enumerate() {
            let col = index % columns;
            let row = index / columns;
            
            if row >= rows {
                break; // Don't exceed grid
            }
            
            let x1 = self.total_bounds.x1 + col * cell_width;
            let y1 = self.total_bounds.y1 + row * cell_height;
            let x2 = x1 + cell_width - 1;
            let y2 = y1 + cell_height - 1;
            
            results.push(Bounds::new(x1, y1, x2, y2));
        }
        
        Ok(results)
    }
    
    /// Calculate flex-based layout  
    fn calculate_flex_layout(&self, direction: FlexDirection) -> Result<Vec<Bounds>, LayoutError> {
        let child_count = self.child_constraints.len();
        if child_count == 0 {
            return Ok(Vec::new());
        }
        
        let mut results = Vec::new();
        
        match direction {
            FlexDirection::Row => {
                let child_width = self.total_bounds.width() / child_count;
                
                for (index, _) in self.child_constraints.iter().enumerate() {
                    let x1 = self.total_bounds.x1 + index * child_width;
                    let x2 = x1 + child_width - 1;
                    let bounds = Bounds::new(x1, self.total_bounds.y1, x2, self.total_bounds.y2);
                    results.push(bounds);
                }
            }
            FlexDirection::Column => {
                let child_height = self.total_bounds.height() / child_count;
                
                for (index, _) in self.child_constraints.iter().enumerate() {
                    let y1 = self.total_bounds.y1 + index * child_height;
                    let y2 = y1 + child_height - 1;
                    let bounds = Bounds::new(self.total_bounds.x1, y1, self.total_bounds.x2, y2);
                    results.push(bounds);
                }
            }
        }
        
        Ok(results)
    }
    
    /// Calculate stack layout (all children same bounds)
    fn calculate_stack_layout(&self) -> Result<Vec<Bounds>, LayoutError> {
        let mut results = Vec::new();
        
        for _ in &self.child_constraints {
            results.push(self.total_bounds);
        }
        
        Ok(results)
    }
    
    /// Calculate space distribution for responsive layouts
    /// Handles percentage-based space allocation
    pub fn distribute_space(&self, percentages: &[f64]) -> Result<Vec<usize>, LayoutError> {
        let total_percent: f64 = percentages.iter().sum();
        if total_percent > 100.01 { // Allow small floating point error
            return Err(LayoutError::PercentageOverflow { total: total_percent });
        }
        
        let available_space = self.total_bounds.width();
        let mut results = Vec::new();
        
        for &percentage in percentages {
            let space = ((percentage / 100.0) * available_space as f64).round() as usize;
            results.push(space);
        }
        
        Ok(results)
    }
    
    /// Calculate minimum required space for all children
    pub fn calculate_minimum_space(&self) -> (usize, usize) {
        if self.child_constraints.is_empty() {
            return (1, 1); // Minimum space for empty layout
        }
        
        match &self.layout_strategy {
            LayoutStrategy::Absolute => {
                // Find bounding box of all children
                let mut min_width = 0;
                let mut min_height = 0;
                
                for constraint in &self.child_constraints {
                    if let Ok(bounds) = self.resolve_input_bounds(constraint) {
                        min_width = min_width.max(bounds.x2 + 1);
                        min_height = min_height.max(bounds.y2 + 1);
                    }
                }
                
                (min_width, min_height)
            }
            LayoutStrategy::Grid { columns, rows } => {
                // Minimum cell size * grid dimensions
                let min_cell_size = (8, 3); // Reasonable minimum for UI elements
                (*columns * min_cell_size.0, *rows * min_cell_size.1)
            }
            LayoutStrategy::Flex { .. } | LayoutStrategy::Stack => {
                // All children need to fit
                let min_child_size = (8, 3);
                (min_child_size.0, min_child_size.1 * self.child_constraints.len())
            }
        }
    }
    
    /// Validate layout fits within available space
    pub fn validate_layout(&self) -> Result<(), LayoutError> {
        let (min_width, min_height) = self.calculate_minimum_space();
        
        if self.total_bounds.width() < min_width {
            return Err(LayoutError::InsufficientWidth {
                available: self.total_bounds.width(),
                required: min_width,
            });
        }
        
        if self.total_bounds.height() < min_height {
            return Err(LayoutError::InsufficientHeight {
                available: self.total_bounds.height(), 
                required: min_height,
            });
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutStrategy {
    /// Absolute positioning using InputBounds
    Absolute,
    /// Grid layout with fixed columns and rows
    Grid { columns: usize, rows: usize },
    /// Flexible layout in one direction
    Flex { direction: FlexDirection },
    /// Stack layout (all children same bounds)
    Stack,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlexDirection {
    Row,    // Arrange children horizontally
    Column, // Arrange children vertically
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Axis {
    X,
    Y,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutError {
    InvalidPercentage(String),
    InvalidCoordinate(String),
    InvalidGridDimensions { columns: usize, rows: usize },
    PercentageOverflow { total: f64 },
    InsufficientWidth { available: usize, required: usize },
    InsufficientHeight { available: usize, required: usize },
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_percentage_parsing() {
        assert_eq!(LayoutDimensions::parse_percentage("50%").unwrap(), 0.5);
        assert_eq!(LayoutDimensions::parse_percentage("100%").unwrap(), 1.0);
        assert_eq!(LayoutDimensions::parse_percentage("0%").unwrap(), 0.0);
        
        // Test clamping
        assert_eq!(LayoutDimensions::parse_percentage("150%").unwrap(), 1.0);
        assert_eq!(LayoutDimensions::parse_percentage("-10%").unwrap(), 0.0);
        
        // Test invalid
        assert!(LayoutDimensions::parse_percentage("not_percent").is_err());
        assert!(LayoutDimensions::parse_percentage("50").is_err());
    }
    
    #[test]
    fn test_percentage_to_absolute() {
        let total_bounds = Bounds::new(0, 0, 100, 50);
        let layout = LayoutDimensions::new(total_bounds);
        
        // 50% of width (101) should be around 50
        let x = layout.percentage_to_absolute(0.5, Axis::X);
        assert_eq!(x, 50);
        
        // 50% of height (51) should be around 25  
        let y = layout.percentage_to_absolute(0.5, Axis::Y);
        assert_eq!(y, 25);
        
        // 100% should be max coordinate
        let x_max = layout.percentage_to_absolute(1.0, Axis::X);
        assert_eq!(x_max, 100); // total - 1
    }
    
    #[test]
    fn test_input_bounds_resolution() {
        let total_bounds = Bounds::new(0, 0, 100, 50);
        let layout = LayoutDimensions::new(total_bounds);
        
        let input = InputBounds {
            x1: "25%".to_string(),
            y1: "20%".to_string(),
            x2: "75%".to_string(),
            y2: "80%".to_string(),
        };
        
        let resolved = layout.resolve_input_bounds(&input).unwrap();
        
        assert_eq!(resolved.x1, 25); // 25% of 100
        assert_eq!(resolved.y1, 10); // 20% of 50
        assert_eq!(resolved.x2, 75); // 75% of 100
        assert_eq!(resolved.y2, 40); // 80% of 50
    }
    
    #[test]
    fn test_anchor_resolution() {
        let total_bounds = Bounds::new(0, 0, 100, 50);
        let layout = LayoutDimensions::new(total_bounds);
        
        let container = Bounds::new(10, 10, 90, 40);
        let target_size = (20, 10);
        
        // Test center anchor
        let bounds = layout.resolve_anchor(&Anchor::Center, target_size, &container);
        
        // Should be centered in container
        // Container: 10,10 to 90,40 = 81x31
        // Target: 20x10
        // Center: 10 + (81-20)/2 = 10 + 30 = 40, 10 + (31-10)/2 = 10 + 10 = 20
        assert_eq!(bounds.x1, 40);
        assert_eq!(bounds.y1, 20);
        assert_eq!(bounds.width(), 20);
        assert_eq!(bounds.height(), 10);
    }
    
    #[test]
    fn test_grid_layout() {
        let total_bounds = Bounds::new(0, 0, 19, 11); // 20x12
        let mut layout = LayoutDimensions::new(total_bounds);
        layout = layout.with_strategy(LayoutStrategy::Grid { columns: 2, rows: 2 });
        
        // Add 4 children
        for i in 0..4 {
            layout.add_child(InputBounds {
                x1: format!("{}%", i * 25),
                y1: "0%".to_string(),
                x2: format!("{}%", (i + 1) * 25),
                y2: "100%".to_string(),
            });
        }
        
        let bounds = layout.calculate_all_child_bounds().unwrap();
        assert_eq!(bounds.len(), 4);
        
        // Each cell should be 10x6
        assert_eq!(bounds[0], Bounds::new(0, 0, 9, 5));   // Top-left
        assert_eq!(bounds[1], Bounds::new(10, 0, 19, 5)); // Top-right 
        assert_eq!(bounds[2], Bounds::new(0, 6, 9, 11));  // Bottom-left
        assert_eq!(bounds[3], Bounds::new(10, 6, 19, 11)); // Bottom-right
    }
    
    #[test]
    fn test_flex_layout() {
        let total_bounds = Bounds::new(0, 0, 29, 9); // 30x10
        let mut layout = LayoutDimensions::new(total_bounds);
        layout = layout.with_strategy(LayoutStrategy::Flex { 
            direction: FlexDirection::Row 
        });
        
        // Add 3 children
        for i in 0..3 {
            layout.add_child(InputBounds {
                x1: format!("{}%", i * 30),
                y1: "0%".to_string(),
                x2: format!("{}%", (i + 1) * 30),
                y2: "100%".to_string(),
            });
        }
        
        let bounds = layout.calculate_all_child_bounds().unwrap();
        assert_eq!(bounds.len(), 3);
        
        // Each should be 10 wide (30/3)
        assert_eq!(bounds[0], Bounds::new(0, 0, 9, 9));   // Left
        assert_eq!(bounds[1], Bounds::new(10, 0, 19, 9)); // Center
        assert_eq!(bounds[2], Bounds::new(20, 0, 29, 9)); // Right
    }
    
    #[test]
    fn test_space_distribution() {
        let total_bounds = Bounds::new(0, 0, 99, 9); // 100 wide
        let layout = LayoutDimensions::new(total_bounds);
        
        let percentages = vec![30.0, 50.0, 20.0];
        let spaces = layout.distribute_space(&percentages).unwrap();
        
        assert_eq!(spaces, vec![30, 50, 20]);
        
        // Test overflow
        let overflow = vec![60.0, 50.0, 20.0]; // 130% total
        assert!(layout.distribute_space(&overflow).is_err());
    }
}