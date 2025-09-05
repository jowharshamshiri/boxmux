use crate::Bounds;
use super::Orientation;

/// ProgressDimensions - Centralizes ALL progress bar and chart mathematical operations
///
/// Eliminates ad hoc progress math like:
/// - filled_width = ((bar_width as f64) * progress).round() as usize
/// - Chart scaling and data mapping scattered throughout chart components
/// - Progress indicator positioning and sizing calculations
///
/// Replaces scattered progress logic from progress_bar.rs, chart_component.rs
#[derive(Debug, Clone)]
pub struct ProgressDimensions {
    /// Available bounds for progress display
    bounds: Bounds,
    /// Progress orientation
    orientation: Orientation,
    /// Current progress value (0.0 to 1.0)
    progress: f64,
    /// Data range for value mapping (min, max)
    data_range: (f64, f64),
}

impl ProgressDimensions {
    /// Create new progress dimensions
    pub fn new(bounds: Bounds, orientation: Orientation) -> Self {
        Self {
            bounds,
            orientation,
            progress: 0.0,
            data_range: (0.0, 1.0),
        }
    }
    
    /// Set current progress (0.0 to 1.0)
    pub fn set_progress(&mut self, progress: f64) {
        self.progress = progress.clamp(0.0, 1.0);
    }
    
    /// Set data range for value mapping
    pub fn set_data_range(&mut self, min: f64, max: f64) {
        self.data_range = (min, max);
    }
    
    /// Calculate filled area size based on progress
    /// Centralizes: filled_width = ((bar_width as f64) * progress).round() as usize
    pub fn calculate_fill_size(&self) -> usize {
        let total_size = match self.orientation {
            Orientation::Horizontal => self.bounds.width(),
            Orientation::Vertical => self.bounds.height(),
        };
        
        if total_size == 0 {
            return 0;
        }
        
        ((total_size as f64) * self.progress).round() as usize
    }
    
    /// Get filled area bounds for drawing
    pub fn get_fill_bounds(&self) -> Bounds {
        let fill_size = self.calculate_fill_size();
        
        match self.orientation {
            Orientation::Horizontal => {
                // Fill from left
                Bounds::new(
                    self.bounds.x1,
                    self.bounds.y1,
                    self.bounds.x1 + fill_size.saturating_sub(1),
                    self.bounds.y2,
                )
            }
            Orientation::Vertical => {
                // Fill from bottom up
                Bounds::new(
                    self.bounds.x1,
                    self.bounds.y2.saturating_sub(fill_size.saturating_sub(1)),
                    self.bounds.x2,
                    self.bounds.y2,
                )
            }
        }
    }
    
    /// Get unfilled area bounds for drawing
    pub fn get_empty_bounds(&self) -> Option<Bounds> {
        let fill_size = self.calculate_fill_size();
        let total_size = match self.orientation {
            Orientation::Horizontal => self.bounds.width(),
            Orientation::Vertical => self.bounds.height(),
        };
        
        if fill_size >= total_size {
            return None; // Completely filled
        }
        
        match self.orientation {
            Orientation::Horizontal => {
                // Empty area to the right of fill
                Some(Bounds::new(
                    self.bounds.x1 + fill_size,
                    self.bounds.y1,
                    self.bounds.x2,
                    self.bounds.y2,
                ))
            }
            Orientation::Vertical => {
                // Empty area above fill
                Some(Bounds::new(
                    self.bounds.x1,
                    self.bounds.y1,
                    self.bounds.x2,
                    self.bounds.y2.saturating_sub(fill_size),
                ))
            }
        }
    }
    
    /// Map a data value to pixel position within bounds
    /// Centralizes chart data-to-pixel mapping
    pub fn map_value_to_pixel(&self, value: f64) -> usize {
        let (min_val, max_val) = self.data_range;
        let range = max_val - min_val;
        
        if range <= 0.0 {
            return match self.orientation {
                Orientation::Horizontal => self.bounds.x1,
                Orientation::Vertical => self.bounds.y1,
            };
        }
        
        let normalized = ((value - min_val) / range).clamp(0.0, 1.0);
        let available_size = match self.orientation {
            Orientation::Horizontal => self.bounds.width(),
            Orientation::Vertical => self.bounds.height(),
        };
        
        let pixel_offset = (normalized * (available_size - 1) as f64).floor() as usize;
        
        match self.orientation {
            Orientation::Horizontal => self.bounds.x1 + pixel_offset,
            Orientation::Vertical => {
                // Vertical charts often display with higher values at the top
                self.bounds.y2 - pixel_offset
            }
        }
    }
    
    /// Map pixel position back to data value
    /// Useful for interactive charts
    pub fn map_pixel_to_value(&self, pixel: usize) -> f64 {
        let (min_val, max_val) = self.data_range;
        let range = max_val - min_val;
        
        let relative_pixel = match self.orientation {
            Orientation::Horizontal => {
                pixel.saturating_sub(self.bounds.x1)
            }
            Orientation::Vertical => {
                // Reverse for vertical (higher Y = lower value)
                self.bounds.y2.saturating_sub(pixel)
            }
        };
        
        let available_size = match self.orientation {
            Orientation::Horizontal => self.bounds.width(),
            Orientation::Vertical => self.bounds.height(),
        };
        
        if available_size <= 1 {
            return min_val;
        }
        
        let normalized = (relative_pixel as f64) / ((available_size - 1) as f64);
        min_val + (normalized * range)
    }
    
    /// Calculate progress bar segment positions for multi-segment bars
    pub fn calculate_segments(&self, segment_count: usize) -> Vec<ProgressSegment> {
        if segment_count == 0 {
            return vec![];
        }
        
        let total_size = match self.orientation {
            Orientation::Horizontal => self.bounds.width(),
            Orientation::Vertical => self.bounds.height(),
        };
        
        let segment_size = total_size / segment_count;
        let remainder = total_size % segment_count;
        let mut segments = Vec::new();
        
        let mut current_pos = match self.orientation {
            Orientation::Horizontal => self.bounds.x1,
            Orientation::Vertical => self.bounds.y1,
        };
        
        for i in 0..segment_count {
            // Distribute remainder across first few segments
            let this_segment_size = if i < remainder {
                segment_size + 1
            } else {
                segment_size
            };
            
            let filled = (self.progress * segment_count as f64) >= (i + 1) as f64;
            let partial_fill = if filled {
                1.0 // Fully filled
            } else {
                // Calculate partial fill for current segment
                let segment_progress = (self.progress * segment_count as f64) - i as f64;
                if segment_progress > 0.0 {
                    segment_progress.min(1.0)
                } else {
                    0.0
                }
            };
            
            let segment_bounds = match self.orientation {
                Orientation::Horizontal => Bounds::new(
                    current_pos,
                    self.bounds.y1,
                    current_pos + this_segment_size - 1,
                    self.bounds.y2,
                ),
                Orientation::Vertical => Bounds::new(
                    self.bounds.x1,
                    current_pos,
                    self.bounds.x2,
                    current_pos + this_segment_size - 1,
                ),
            };
            
            segments.push(ProgressSegment {
                bounds: segment_bounds,
                fill_ratio: partial_fill,
                is_complete: filled,
                segment_index: i,
            });
            
            current_pos += this_segment_size;
        }
        
        segments
    }
    
    /// Calculate chart axis tick positions
    /// Centralizes chart axis calculation
    pub fn calculate_axis_ticks(&self, tick_count: usize) -> Vec<AxisTick> {
        if tick_count <= 1 {
            return vec![];
        }
        
        let (min_val, max_val) = self.data_range;
        let value_step = (max_val - min_val) / (tick_count - 1) as f64;
        let mut ticks = Vec::new();
        
        for i in 0..tick_count {
            let value = min_val + (i as f64 * value_step);
            let pixel = self.map_value_to_pixel(value);
            
            ticks.push(AxisTick {
                value,
                pixel_position: pixel,
                label: format!("{:.1}", value),
                is_major: i % 5 == 0, // Every 5th tick is major
            });
        }
        
        ticks
    }
    
    /// Calculate data point positions for line charts
    pub fn calculate_data_points(&self, data: &[f64]) -> Vec<DataPoint> {
        if data.is_empty() {
            return vec![];
        }
        
        let mut points = Vec::new();
        let x_step = if data.len() <= 1 {
            0.0
        } else {
            (self.bounds.width() - 1) as f64 / (data.len() - 1) as f64
        };
        
        for (i, &value) in data.iter().enumerate() {
            let x = self.bounds.x1 + (i as f64 * x_step).round() as usize;
            let y = self.map_value_to_pixel(value);
            
            points.push(DataPoint {
                value,
                x,
                y,
                index: i,
            });
        }
        
        points
    }
    
    /// Calculate bar chart bar positions and heights
    pub fn calculate_bars(&self, data: &[f64]) -> Vec<ChartBar> {
        if data.is_empty() {
            return vec![];
        }
        
        let bar_count = data.len();
        let total_width = self.bounds.width();
        let bar_width = total_width / bar_count;
        let remainder = total_width % bar_count;
        
        let mut bars = Vec::new();
        let mut current_x = self.bounds.x1;
        
        for (i, &value) in data.iter().enumerate() {
            // Distribute remainder pixels across first few bars
            let this_bar_width = if i < remainder {
                bar_width + 1
            } else {
                bar_width
            };
            
            let bar_height_pixels = self.calculate_bar_height(value);
            let bar_top_y = self.bounds.y2.saturating_sub(bar_height_pixels.saturating_sub(1));
            
            let bar_bounds = Bounds::new(
                current_x,
                bar_top_y,
                current_x + this_bar_width - 1,
                self.bounds.y2,
            );
            
            bars.push(ChartBar {
                value,
                bounds: bar_bounds,
                height_pixels: bar_height_pixels,
                index: i,
            });
            
            current_x += this_bar_width;
        }
        
        bars
    }
    
    /// Calculate bar height based on value and available space
    fn calculate_bar_height(&self, value: f64) -> usize {
        let (min_val, max_val) = self.data_range;
        let range = max_val - min_val;
        let available_height = self.bounds.height();
        
        if range <= 0.0 || available_height == 0 {
            return if value > min_val { available_height } else { 0 };
        }
        
        let normalized = ((value - min_val) / range).clamp(0.0, 1.0);
        (normalized * available_height as f64).round() as usize
    }
    
    /// Calculate sparkline positions (compact inline charts)
    pub fn calculate_sparkline(&self, data: &[f64]) -> Vec<SparklinePoint> {
        if data.is_empty() {
            return vec![];
        }
        
        // For sparklines, use available width directly - one point per pixel
        let width = self.bounds.width();
        let mut points = Vec::new();
        
        if data.len() <= width {
            // If we have fewer or equal data points than pixels, distribute evenly
            let x_step = if data.len() == 1 { 0.0 } else { (width - 1) as f64 / (data.len() - 1) as f64 };
            
            for (i, &value) in data.iter().enumerate() {
                let x = (i as f64 * x_step).round() as usize;
                let normalized_y = self.map_value_to_pixel(value);
                
                points.push(SparklinePoint {
                    x: self.bounds.x1 + x,
                    y: normalized_y,
                    value,
                    data_index: i,
                });
            }
            
            // Fill remaining pixels with interpolated values if needed
            if data.len() > 1 {
                for x_pos in 0..width {
                    let needs_point = !points.iter().any(|p| p.x == self.bounds.x1 + x_pos);
                    if needs_point {
                        // Interpolate value for this position
                        let data_pos = (x_pos as f64 / (width - 1) as f64) * (data.len() - 1) as f64;
                        let data_index = data_pos.floor() as usize;
                        let next_index = (data_index + 1).min(data.len() - 1);
                        
                        let frac = data_pos - data_index as f64;
                        let value = data[data_index] + frac * (data[next_index] - data[data_index]);
                        let normalized_y = self.map_value_to_pixel(value);
                        
                        points.push(SparklinePoint {
                            x: self.bounds.x1 + x_pos,
                            y: normalized_y,
                            value,
                            data_index,
                        });
                    }
                }
            }
        } else {
            // More data points than pixels - sample data
            for x_pos in 0..width {
                let data_pos = (x_pos as f64 / (width - 1) as f64) * (data.len() - 1) as f64;
                let data_index = data_pos.round() as usize;
                let value = data[data_index];
                let normalized_y = self.map_value_to_pixel(value);
                
                points.push(SparklinePoint {
                    x: self.bounds.x1 + x_pos,
                    y: normalized_y,
                    value,
                    data_index,
                });
            }
        }
        
        // Sort by x position for consistency
        points.sort_by_key(|p| p.x);
        points
    }
    
    /// Validate progress dimensions are reasonable
    pub fn validate(&self) -> Result<(), ProgressDimensionError> {
        // Check for degenerate bounds - when x1 == x2 and y1 == y2, we have zero useful area
        if self.bounds.x1 == self.bounds.x2 && self.bounds.y1 == self.bounds.y2 {
            return Err(ProgressDimensionError::ZeroDimensions);
        }
        
        let (min_val, max_val) = self.data_range;
        if min_val >= max_val {
            return Err(ProgressDimensionError::InvalidDataRange { min: min_val, max: max_val });
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProgressSegment {
    pub bounds: Bounds,
    pub fill_ratio: f64,    // 0.0 to 1.0
    pub is_complete: bool,
    pub segment_index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AxisTick {
    pub value: f64,
    pub pixel_position: usize,
    pub label: String,
    pub is_major: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataPoint {
    pub value: f64,
    pub x: usize,
    pub y: usize,
    pub index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChartBar {
    pub value: f64,
    pub bounds: Bounds,
    pub height_pixels: usize,
    pub index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SparklinePoint {
    pub x: usize,
    pub y: usize,
    pub value: f64,
    pub data_index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProgressDimensionError {
    ZeroDimensions,
    InvalidDataRange { min: f64, max: f64 },
    InvalidProgress { value: f64 },
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fill_size_calculation() {
        let bounds = Bounds::new(0, 0, 19, 5); // 20 wide
        let mut progress_dims = ProgressDimensions::new(bounds, Orientation::Horizontal);
        
        progress_dims.set_progress(0.5); // 50%
        let fill_size = progress_dims.calculate_fill_size();
        assert_eq!(fill_size, 10); // 50% of 20
        
        progress_dims.set_progress(0.75); // 75%
        let fill_size = progress_dims.calculate_fill_size();
        assert_eq!(fill_size, 15); // 75% of 20
    }
    
    #[test]
    fn test_fill_bounds() {
        let bounds = Bounds::new(5, 5, 14, 7); // 10 wide, starts at (5,5)
        let mut progress_dims = ProgressDimensions::new(bounds, Orientation::Horizontal);
        
        progress_dims.set_progress(0.6); // 60%
        let fill_bounds = progress_dims.get_fill_bounds();
        
        assert_eq!(fill_bounds.x1, 5); // Same start
        assert_eq!(fill_bounds.x2, 10); // 5 + 6 - 1 (60% of 10 = 6)
        assert_eq!(fill_bounds.y1, 5);
        assert_eq!(fill_bounds.y2, 7);
    }
    
    #[test]
    fn test_empty_bounds() {
        let bounds = Bounds::new(0, 0, 9, 3); // 10 wide
        let mut progress_dims = ProgressDimensions::new(bounds, Orientation::Horizontal);
        
        progress_dims.set_progress(0.3); // 30%
        let empty_bounds = progress_dims.get_empty_bounds().unwrap();
        
        assert_eq!(empty_bounds.x1, 3); // Start after 30% fill
        assert_eq!(empty_bounds.x2, 9); // End of original bounds
        
        // Test fully filled - should have no empty area
        progress_dims.set_progress(1.0);
        assert!(progress_dims.get_empty_bounds().is_none());
    }
    
    #[test]
    fn test_value_to_pixel_mapping() {
        let bounds = Bounds::new(0, 0, 99, 19); // 100 wide, 20 tall
        let mut progress_dims = ProgressDimensions::new(bounds, Orientation::Horizontal);
        progress_dims.set_data_range(0.0, 100.0); // 0 to 100 data range
        
        // Value 50 should map to middle (pixel 50)
        let pixel = progress_dims.map_value_to_pixel(50.0);
        assert_eq!(pixel, 49); // 0 + (50/100) * 99 ≈ 49
        
        // Value 0 should map to start
        let pixel = progress_dims.map_value_to_pixel(0.0);
        assert_eq!(pixel, 0);
        
        // Value 100 should map to end
        let pixel = progress_dims.map_value_to_pixel(100.0);
        assert_eq!(pixel, 99);
    }
    
    #[test]
    fn test_pixel_to_value_mapping() {
        let bounds = Bounds::new(0, 0, 99, 19); // 100 wide
        let mut progress_dims = ProgressDimensions::new(bounds, Orientation::Horizontal);
        progress_dims.set_data_range(0.0, 100.0);
        
        // Pixel 49 should map to value 50 (approximately)
        let value = progress_dims.map_pixel_to_value(49);
        assert!((value - 49.5).abs() < 1.0); // Should be close to 49.5
        
        // Pixel 0 should map to value 0
        let value = progress_dims.map_pixel_to_value(0);
        assert!((value - 0.0).abs() < 0.1);
        
        // Pixel 99 should map to value 100
        let value = progress_dims.map_pixel_to_value(99);
        assert!((value - 100.0).abs() < 0.1);
    }
    
    #[test]
    fn test_segments_calculation() {
        let bounds = Bounds::new(0, 0, 9, 2); // 10 wide
        let mut progress_dims = ProgressDimensions::new(bounds, Orientation::Horizontal);
        progress_dims.set_progress(0.35); // 35% progress
        
        let segments = progress_dims.calculate_segments(5); // 5 segments
        assert_eq!(segments.len(), 5);
        
        // Each segment should be 2 wide (10/5)
        for segment in &segments {
            assert_eq!(segment.bounds.width(), 2);
        }
        
        // First segment should be complete (35% > 20%)
        assert!(segments[0].is_complete);
        
        // Second segment should be partially filled  
        assert!(!segments[1].is_complete);
        assert!(segments[1].fill_ratio > 0.0 && segments[1].fill_ratio < 1.0);
        
        // Later segments should be empty
        assert_eq!(segments[3].fill_ratio, 0.0);
        assert_eq!(segments[4].fill_ratio, 0.0);
    }
    
    #[test]
    fn test_axis_ticks() {
        let bounds = Bounds::new(0, 0, 99, 19);
        let mut progress_dims = ProgressDimensions::new(bounds, Orientation::Horizontal);
        progress_dims.set_data_range(0.0, 100.0);
        
        let ticks = progress_dims.calculate_axis_ticks(6); // 6 ticks
        assert_eq!(ticks.len(), 6);
        
        // First tick should be at min value
        assert_eq!(ticks[0].value, 0.0);
        assert_eq!(ticks[0].pixel_position, 0);
        
        // Last tick should be at max value
        assert_eq!(ticks[5].value, 100.0);
        assert_eq!(ticks[5].pixel_position, 99);
        
        // Ticks should be evenly spaced
        let expected_step = 100.0 / 5.0; // (max - min) / (count - 1)
        for i in 0..ticks.len() {
            let expected_value = i as f64 * expected_step;
            assert!((ticks[i].value - expected_value).abs() < 0.1);
        }
    }
    
    #[test]
    fn test_bar_chart_calculation() {
        let bounds = Bounds::new(0, 5, 9, 15); // 10 wide, 11 tall
        let mut progress_dims = ProgressDimensions::new(bounds, Orientation::Vertical);
        progress_dims.set_data_range(0.0, 10.0);
        
        let data = vec![5.0, 10.0, 2.5]; // 3 bars
        let bars = progress_dims.calculate_bars(&data);
        
        assert_eq!(bars.len(), 3);
        
        // Each bar should be roughly 3-4 wide (10/3 with remainder distribution)
        assert!(bars[0].bounds.width() >= 3 && bars[0].bounds.width() <= 4);
        
        // Tallest bar (value 10.0) should have maximum height
        let max_bar = bars.iter().max_by(|a, b| a.height_pixels.cmp(&b.height_pixels)).unwrap();
        assert_eq!(max_bar.value, 10.0);
        assert_eq!(max_bar.height_pixels, 11); // Full height
        
        // Bar with value 5.0 should be half height
        let half_bar = bars.iter().find(|b| b.value == 5.0).unwrap();
        assert_eq!(half_bar.height_pixels, 6); // Approximately half of 11
    }
    
    #[test]
    fn test_sparkline_calculation() {
        let bounds = Bounds::new(0, 0, 9, 3); // 10 wide
        let mut progress_dims = ProgressDimensions::new(bounds, Orientation::Horizontal);
        progress_dims.set_data_range(0.0, 10.0);
        
        let data = vec![1.0, 5.0, 8.0, 3.0, 9.0]; // 5 data points, 10 pixel width
        let points = progress_dims.calculate_sparkline(&data);
        
        assert_eq!(points.len(), 10); // One point per pixel
        
        // Points should span the full width
        assert_eq!(points[0].x, 0);
        assert_eq!(points[9].x, 9);
        
        // Values should be properly mapped
        for point in &points {
            assert!(point.value >= 0.0 && point.value <= 10.0);
        }
    }
    
    #[test]
    fn test_validation() {
        let bounds = Bounds::new(0, 0, 10, 5);
        let mut progress_dims = ProgressDimensions::new(bounds, Orientation::Horizontal);
        
        // Valid configuration should pass
        progress_dims.set_data_range(0.0, 100.0);
        assert!(progress_dims.validate().is_ok());
        
        // Invalid data range should fail
        progress_dims.set_data_range(100.0, 50.0); // min > max
        assert!(progress_dims.validate().is_err());
        
        // Zero dimensions should fail
        let zero_bounds = Bounds::new(0, 0, 0, 0); // Zero width/height
        let zero_progress_dims = ProgressDimensions::new(zero_bounds, Orientation::Horizontal);
        assert!(zero_progress_dims.validate().is_err());
    }
}