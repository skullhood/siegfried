use std::collections::{HashMap};
use std::ops::Mul;
use crate::position::{Move, Position};
use crate::types::{GameStateConstants, Side, SideConstants, GameState};

use rand::seq::SliceRandom;
use rayon::prelude::*;

#[derive(PartialEq, Clone, Copy)]
pub struct ExpandStyle(pub u8);

impl ExpandStyle{
    pub const DEFAULT: ExpandStyle = ExpandStyle(0);
    pub const RANDOM: ExpandStyle = ExpandStyle(1);
}

fn calculate_all_moves_to_expand(total_moves: usize) -> usize{

    let moves_to_expand = 4 * (total_moves as f64).sqrt() as usize;

    if moves_to_expand > total_moves{
        return total_moves;
    }
    return  moves_to_expand;
}

fn calculate_moves_to_expand(total_moves: usize) -> usize{

    let moves_to_expand = (total_moves as f64).sqrt() as usize + 1;

    if moves_to_expand > total_moves{
        return total_moves;
    }
    return  moves_to_expand;
}


#[derive(Clone)]
pub struct Node{
    pub parent_move: Option<Move>,
    pub position: Position,
    pub available_moves: Vec<Move>,
    pub score: Option<f32>,
    pub game_state: GameState,
    pub depth: u8,
}

pub struct PositionTree{
    pub root: usize,
    pub parent: HashMap<usize, usize>,
    pub children: HashMap<usize, Vec<usize>>,
    pub values: HashMap<usize, Node>,
    pub depth: u8,
}

impl PositionTree{
    pub fn new(position: Position) -> PositionTree{
        let mut tree = PositionTree{
            root: 0,
            parent: HashMap::new(),
            children: HashMap::new(),
            values: HashMap::new(),
            depth: 0,
        };
        let eval = position.evaluate();
        tree.values.insert(0, Node{
            parent_move: None,
            position,
            available_moves: eval.moves,
            score: Some(0.0),
            game_state: GameState::ONGOING,
            depth: 0
        });
        tree
    }

    pub fn get_node(&self, index: usize) -> &Node{
        return self.values.get(&index).unwrap();
    }

    pub fn get_node_mut(&mut self, index: usize) -> &mut Node{
        return self.values.get_mut(&index).unwrap();
    }

    pub fn get_parent(&self, index: usize) -> Option<usize>{
        return self.parent.get(&index).cloned();
    }

    pub fn get_children(&self, index: usize) -> Option<&Vec<usize>>{
        return self.children.get(&index);
    }

    pub fn get_children_mut(&mut self, index: usize) -> Option<&mut Vec<usize>>{
        return self.children.get_mut(&index);
    }

    pub fn get_available_moves(&self, index: usize) -> Vec<Move>{
        return self.get_node(index).available_moves.clone();
    }

    pub fn get_score(&self, index: usize) -> Option<f32>{
        return self.get_node(index).score.clone();
    }

    pub fn get_game_state(&self, index: usize) -> GameState{
        return GameState(self.get_node(index).game_state.0);
    }
        /*
        pub parent_move: Option<Move>,
        pub position: Position,
        pub available_moves: Vec<Move>,
        pub score: i32,
        */

    fn get_node_children(&self, index: usize) -> Vec<Node>{
        let node = self.get_node(index);
        node.available_moves.clone().into_par_iter().map(|m| {
            let new_position = node.position.make_move(m);
            let eval = new_position.evaluate();
            Node{
                parent_move: Some(m.clone()),
                position: new_position,
                available_moves: eval.moves,
                score: eval.score,
                game_state: eval.game_state,
                depth: node.depth + 1
            }
        }).collect::<Vec<Node>>()
    }

    fn expand_node(&mut self, index: usize, expand_style: ExpandStyle, playing_side: Side){
        //sort moves by score descending
        let mut children = self.get_node_children(index);

        let playing_multiplier = if playing_side == Side::WHITE{
            -1.0
        }else{
            1.0
        };

        if expand_style == ExpandStyle::DEFAULT{
            children.par_sort_by_key(|n|  if n.score.is_some(){if n.position.side_to_move == Side::WHITE{(playing_multiplier * n.score.unwrap() * 1000.0) as i32}else{(playing_multiplier * n.score.unwrap() * 1000.0) as i32}}else{if playing_side == n.position.side_to_move{-1000}else{1000}});
        }
        else if expand_style == ExpandStyle::RANDOM{
            children.shuffle(&mut rand::thread_rng());
        }

        let mut depth = 0;
        let mut child_indices = Vec::new();
        let mut scores: Vec<f32> = Vec::new();
        for child in children{
            let child_index = self.values.len();
            child_indices.push(child_index);
            self.values.insert(child_index, child);
            self.parent.insert(child_index, index);
            let child_score = self.get_node(child_index).score;
            if child_score.is_some(){
                scores.push(child_score.unwrap());
            }
            depth = self.get_node(child_index).depth;
        }
        self.depth = depth;
        self.children.insert(index, child_indices);
        //update score of index node to be the score of the average of the children
        let mut node = self.get_node_mut(index);
        node.score = Some(scores.par_iter().sum::<f32>() / scores.len() as f32);   
    }

    fn get_nodes_to_expand(&self, index: usize) -> Vec<usize>{
        let mut nodes_to_expand = Vec::new();

        //check if index node is end node
        if self.get_game_state(index) == GameState::CHECKMATE || self.get_game_state(index) == GameState::DRAW{
            return nodes_to_expand;
        }

        let moves_to_expand = calculate_moves_to_expand(self.values.len());

        //get all children
        let children = self.get_children(index).unwrap().clone();

        //get all children that are in gamestate CHECK
        let checks = children.par_iter().filter(|c| self.get_game_state(**c) == GameState::CHECK).collect::<Vec<&usize>>();

        //get the first moves_to_expand children that are ongoing
        let mut non_checks = children.par_iter().filter(|c| self.get_game_state(**c) == GameState::ONGOING).collect::<Vec<&usize>>();
        non_checks.truncate(moves_to_expand);

        //add all checks and non_checks to nodes_to_expand
        nodes_to_expand.extend(checks);
        nodes_to_expand.extend(non_checks);

        return nodes_to_expand;
    }

    fn get_all_nodes_to_expand(&self) -> Vec<usize>{
        let mut nodes_to_expand = Vec::new();

        //get all nodes at depth that are checks
        let checks_at_depth = self.values.par_iter().filter(|(i, n)| n.depth == self.depth && self.get_game_state(**i) == GameState::CHECK).map(|(i, _n)| i).collect::<Vec<&usize>>();

        //get all nodes at depth that are not checks
        let mut nodes_at_depth = self.values.par_iter().filter(|(i, n)| n.depth == self.depth && self.get_game_state(**i) == GameState::ONGOING).map(|(i, _n)| i).collect::<Vec<&usize>>();
        let nodes_to_evaluate = calculate_all_moves_to_expand(nodes_at_depth.len());

        nodes_at_depth.truncate(nodes_to_evaluate);
        //add all nodes at depth that are checks to nodes_to_expand

        nodes_to_expand.extend(checks_at_depth);
        nodes_to_expand.extend(nodes_at_depth);

        return nodes_to_expand;
    }

    fn backpropagate(&mut self, parents: Vec<usize>){
        let mut current_parents = parents;

        while current_parents.len() > 0{

            let children_total: HashMap<usize, usize> = current_parents.par_iter().map(|p| {
                let children = self.get_children(*p).unwrap();
                (*p, children.len())
            }).collect();

            let children_scores: HashMap<usize, Vec<f32>> = current_parents.par_iter().map(|p| {
                let children = self.get_children(*p).unwrap();
                let scores = children.par_iter().map(|c| self.get_score(*c).unwrap()).collect::<Vec<f32>>();
                (*p, scores)
            }).collect();

            let mut new_parents: Vec<usize> = Vec::new();

            for parent in current_parents{
                let total = children_total.get(&parent).unwrap();
                let scores = children_scores.get(&parent).unwrap();
                let mut node = self.get_node_mut(parent);
                node.score = Some(scores.par_iter().sum::<f32>() / *total as f32);
                let grandparent_wrapped = &self.get_parent(parent);
                if grandparent_wrapped.is_some(){
                    let grandparent = grandparent_wrapped.unwrap();
                    if !new_parents.contains(&grandparent){
                        new_parents.push(grandparent);
                    }
                }
            }

            current_parents = new_parents;
        }
    }


    pub fn expand_to_depth(&mut self, depth: u8, expand_style: ExpandStyle, playing_side: Side) -> Vec<Move>{

        let mut moves: Vec<Move> = Vec::new();

        while self.depth < depth{
            let nodes_to_expand = self.get_all_nodes_to_expand();
            let mut parents_for_backpropagation = Vec::new();

            for node in nodes_to_expand{

                self.expand_node(node, expand_style, playing_side);
                
                let parent_node = self.get_parent(node);

                //if not in parents_for_backpropagation, add it
                
                if parent_node.is_some(){
                    let parent = &parent_node.unwrap();  
                    if !parents_for_backpropagation.contains(parent){
                        parents_for_backpropagation.push(*parent);
                    }
                }
            }

            self.backpropagate(parents_for_backpropagation);

            println!("At depth {}", self.depth);
        }

        //get all children of root
        let mut children = self.get_children(0).unwrap().clone();
        
        let side_multiplier = if playing_side == Side::WHITE {1.0} else {-1.0};
        children.sort_by(|a, b| self.get_score(*b).unwrap().mul(side_multiplier).partial_cmp(&self.get_score(*a).unwrap().mul(side_multiplier)).unwrap());

        children.truncate(calculate_all_moves_to_expand(children.len()));

        for child in children{
            let move_to_add = self.get_node(child).parent_move.unwrap();
            moves.push(move_to_add);
        }
        //sort children by score
            
        return moves;
    }

    //disgustingly inefficient
    pub fn expand_to_depth_v2(&mut self, depth: u8, expand_style: ExpandStyle, playing_side: Side) -> Vec<(Move, f32)>{

        let mut move_scores: Vec<(Move, f32)> = Vec::new();

        while self.depth < depth{
            let nodes_to_expand = self.get_all_nodes_to_expand();
            let mut parents_for_backpropagation = Vec::new();

            for node in nodes_to_expand{
                
                let parent_node = self.get_parent(node);

                if parent_node.is_some(){
                    let parent = &parent_node.unwrap();
                    if !parents_for_backpropagation.contains(parent){
                        parents_for_backpropagation.push(*parent);
                    }
                }
                else{
                    self.expand_node(node, expand_style, playing_side);
                }
            }

            let parents_for_expanding_children = parents_for_backpropagation.clone();

            for parent in parents_for_expanding_children{
                let nodes_to_expand = self.get_nodes_to_expand(parent);

                for node in nodes_to_expand{
                    self.expand_node(node, expand_style, playing_side);
                }
            }


            self.backpropagate(parents_for_backpropagation);

            println!("At depth {}", self.depth);
        }

        //get all children of root
        let mut children = self.get_children(0).unwrap().clone();
        
        let side_multiplier = if playing_side == Side::WHITE {1.0} else {-1.0};
        children.sort_by(|a, b| self.get_score(*b).unwrap().mul(side_multiplier).partial_cmp(&self.get_score(*a).unwrap().mul(side_multiplier)).unwrap());

        children.truncate(calculate_all_moves_to_expand(children.len()));

        for child in children{
            let move_to_add = self.get_node(child).parent_move.unwrap();
            let score = self.get_score(child).unwrap();
            move_scores.push((move_to_add, score));
        }
        //sort children by score
            
        return move_scores;
    }
    
}
