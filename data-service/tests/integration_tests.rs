//! Integration tests for Data Service API endpoints
//!
//! These tests use mock repositories to test the API handlers in isolation
//! without requiring a real database or Redis connection.

mod common;
mod staff_api_tests;
mod group_api_tests;
mod membership_api_tests;

